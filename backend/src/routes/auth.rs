use axum::{Json, extract::FromRequestParts, http::request::Parts};
use ocas_auth::{AppRoleLoader, AuthError, Claims};
use uuid::Uuid;

use crate::AppState;

/// local user representation bridging ocas identity with cwdb roles
pub struct CwdbUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl AppRoleLoader for AppState {
    type Role = CwdbUser;
    type Error = AuthError;

    async fn loadrole(&self, userid: Uuid) -> Result<CwdbUser, AuthError> {
        let existing = sqlx::query_as!(
            CwdbUser,
            "SELECT id, email, role FROM users WHERE id = $1",
            userid
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

        if let Some(user) = existing {
            return Ok(user);
        }

        Err(AuthError::Unauthorized("user not found".into()))
    }
}

/// upserts user on first login, assigns admin role if email is in ADMIN_EMAILS
pub async fn upsertuser(state: &AppState, claims: &Claims) -> Result<CwdbUser, AuthError> {
    let existing = sqlx::query_as!(
        CwdbUser,
        "SELECT id, email, role FROM users WHERE id = $1",
        claims.sub
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

    if let Some(user) = existing {
        if user.email != claims.email {
            sqlx::query!("UPDATE users SET email = $1 WHERE id = $2", claims.email, claims.sub)
                .execute(&state.db)
                .await
                .map_err(|e| AuthError::Unauthorized(e.to_string()))?;
        }
        return Ok(user);
    }

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
        let has_token = parts
            .headers
            .get("authorization")
            .map(|v| v.to_str().ok().is_some_and(|s| s.starts_with("Bearer ")))
            .unwrap_or(false)
            || parts
                .headers
                .get("cookie")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|c| c.contains("ocas_access="));

        if !has_token {
            return Ok(OptionalAuth::Anonymous);
        }

        let claims = Claims::from_request_parts(parts, state).await?;
        let user = upsertuser(state, &claims).await?;
        Ok(OptionalAuth::Authenticated(user))
    }
}

/// GET /api/auth/mock/login?email=foo@bar.com
#[cfg(feature = "mock")]
pub async fn mock_login(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let email = params.get("email").cloned().unwrap_or_else(|| "student@iiit.ac.in".to_string());
    
    // Create a mock token using faculty helper so it can take the custom email directly
    let mut mock_claims = ocas_auth::mock::faculty(&email);
    // Use a deterministic UUID so logging in multiple times with the same email doesn't create new conflicting users
    mock_claims.claims.sub = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, email.as_bytes());
    let token = mock_claims.token();
    
    // Set it in cookie and redirect to frontend
    let mut resp = axum::response::Redirect::to("http://localhost:5173").into_response();
    let headers = resp.headers_mut();
    
    let cookie = format!("ocas_access={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=3600", token);
    headers.append(
        axum::http::header::SET_COOKIE,
        cookie.parse().unwrap(),
    );
    
    resp
}

#[cfg(not(feature = "mock"))]
pub async fn mock_login() -> impl axum::response::IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "Mock login not enabled")
}


