use axum::{Json, extract::FromRequestParts, http::request::Parts, response::IntoResponse};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::AppState;

pub struct Claims {
    pub sub: Uuid,
    pub email: String,
}

pub enum AuthError {
    Unauthorized(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let AuthError::Unauthorized(msg) = self;
        (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": msg }))).into_response()
    }
}

/// local user representation bridging auth identity with cwdb roles
pub struct CwdbUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

#[cfg(feature = "ocas")]
use ocas_auth::{AppRoleLoader, AuthError as OcasAuthError, Claims as OcasClaims};

#[cfg(feature = "ocas")]
impl AppRoleLoader for AppState {
    type Role = CwdbUser;
    type Error = OcasAuthError;

    async fn loadrole(&self, userid: Uuid) -> Result<CwdbUser, OcasAuthError> {
        let existing = sqlx::query_as!(
            CwdbUser,
            "SELECT id, email, role FROM users WHERE id = $1",
            userid
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| OcasAuthError::Unauthorized(e.to_string()))?;

        if let Some(user) = existing {
            return Ok(user);
        }

        Err(OcasAuthError::Unauthorized("user not found".into()))
    }
}

#[cfg(feature = "ocas")]
impl FromRequestParts<AppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let ocas_claims = OcasClaims::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                if let OcasAuthError::Unauthorized(msg) = e {
                    AuthError::Unauthorized(msg)
                } else {
                    AuthError::Unauthorized("unauthorized".into())
                }
            })?;
        Ok(Claims { sub: ocas_claims.sub, email: ocas_claims.email })
    }
}

#[cfg(feature = "cas")]
impl FromRequestParts<AppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let cookies = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = cookies
            .split(';')
            .find_map(|c| c.trim().strip_prefix("cwdb_session="))
            .ok_or_else(|| AuthError::Unauthorized("no session".into()))?;

        crate::routes::cas::validate_session(token, &state.config.session_secret)
    }
}

/// upserts user on first login, assigns admin role if email is in ADMIN_EMAILS
pub async fn upsertuser(state: &AppState, claims: &Claims) -> Result<CwdbUser, AuthError> {
    let role = if state
        .config
        .admin_emails
        .iter()
        .any(|e| e.eq_ignore_ascii_case(&claims.email))
    {
        "admin"
    } else {
        "user"
    };

    let existing = sqlx::query_as!(
        CwdbUser,
        "SELECT id, email, role FROM users WHERE id = $1",
        claims.sub
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

    if let Some(user) = existing {
        if user.email != claims.email || user.role != role {
            let user = sqlx::query_as!(
                CwdbUser,
                "UPDATE users SET email = $1, role = $2 WHERE id = $3 RETURNING id, email, role",
                claims.email,
                role,
                claims.sub
            )
            .fetch_one(&state.db)
            .await
            .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

            return Ok(user);
        }

        return Ok(user);
    }

    let user = sqlx::query_as!(
        CwdbUser,
        "INSERT INTO users (id, email, role) VALUES ($1, $2, $3) RETURNING id, email, role",
        claims.sub,
        claims.email,
        role
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

    Ok(user)
}

/// GET /api/auth/me — returns current user info
pub async fn me(
    claims: Claims,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let user = upsertuser(&state, &claims).await?;
    Ok(Json(serde_json::json!({
        "id": user.id,
        "email": user.email,
        "role": user.role,
    })))
}

/// optional auth extractor — returns None when no token, 401 on bad token
pub enum OptionalAuth {
    Authenticated(CwdbUser),
    Anonymous,
}

impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let cookie_str = parts
            .headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        #[cfg(feature = "ocas")]
        let has_token = parts
            .headers
            .get("authorization")
            .map(|v| v.to_str().ok().is_some_and(|s| s.starts_with("Bearer ")))
            .unwrap_or(false)
            || cookie_str.contains("ocas_access=");

        #[cfg(feature = "cas")]
        let has_token = cookie_str.contains("cwdb_session=");

        if !has_token {
            return Ok(OptionalAuth::Anonymous);
        }

        let claims = match Claims::from_request_parts(parts, state).await {
            Ok(c) => c,
            Err(_) => return Ok(OptionalAuth::Anonymous),
        };
        let user = match upsertuser(state, &claims).await {
            Ok(u) => u,
            Err(_) => return Ok(OptionalAuth::Anonymous),
        };
        Ok(OptionalAuth::Authenticated(user))
    }
}

/// GET /api/auth/mock/login?email=foo@bar.com
#[cfg(all(feature = "mock", feature = "ocas"))]
pub async fn mock_login(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let email = params.get("email").cloned().unwrap_or_else(|| "student@iiit.ac.in".to_string());

    let mut mock_claims = ocas_auth::mock::faculty(&email);
    mock_claims.claims.sub = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, email.as_bytes());
    mock_claims.claims.exp = i64::MAX;
    let token = mock_claims.token();

    let mut resp = axum::response::Redirect::to("http://localhost:5173").into_response();
    let cookie = format!("ocas_access={}; HttpOnly; SameSite=Lax; Path=/", token);
    resp.headers_mut().append(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    resp
}

#[cfg(all(feature = "mock", feature = "cas"))]
pub async fn mock_login(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let email = params.get("email").cloned().unwrap_or_else(|| "student@iiit.ac.in".to_string());
    let sub = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, email.as_bytes());
    let token = crate::routes::cas::issue_session(sub, &email, &state.config.session_secret);

    let mut resp = axum::response::Redirect::to(&state.config.frontend_url).into_response();
    let cookie = format!("cwdb_session={}; HttpOnly; SameSite=Lax; Path=/", token);
    resp.headers_mut().append(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    resp
}

#[cfg(not(feature = "mock"))]
pub async fn mock_login() -> impl axum::response::IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "Mock login not enabled")
}
