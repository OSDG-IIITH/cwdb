use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::{AppState, auth};

const SESSION_COOKIE: &str = "cwdb_session";
const OAUTH_STATE_COOKIE: &str = "cwdb_oauth_state";
const OAUTH_NONCE_COOKIE: &str = "cwdb_oauth_nonce";
const SESSION_DURATION_DAYS: i64 = 7;
const FRONTEND_URL: &str = "http://localhost:5173";

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn login(State(state): State<AppState>, cookies: Cookies) -> Redirect {
    let secure = should_use_secure_cookies(&state);

    let oauth_state = Uuid::new_v4().to_string();
    let oauth_nonce = Uuid::new_v4().to_string();

    cookies.add(build_cookie(
        OAUTH_STATE_COOKIE,
        oauth_state.clone(),
        secure,
        time::Duration::minutes(10),
    ));
    cookies.add(build_cookie(
        OAUTH_NONCE_COOKIE,
        oauth_nonce.clone(),
        secure,
        time::Duration::minutes(10),
    ));

    let url = auth::build_login_url(&state.config, &oauth_state, &oauth_nonce);
    Redirect::temporary(&url)
}

pub async fn callback(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let secure = should_use_secure_cookies(&state);

    let expected_state = cookies
        .get(OAUTH_STATE_COOKIE)
        .map(|cookie| cookie.value().to_string());
    let expected_nonce = cookies
        .get(OAUTH_NONCE_COOKIE)
        .map(|cookie| cookie.value().to_string());

    clear_cookie(&cookies, OAUTH_STATE_COOKIE, secure);
    clear_cookie(&cookies, OAUTH_NONCE_COOKIE, secure);

    if expected_state.as_deref() != Some(query.state.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid OAuth state" })),
        )
            .into_response();
    }

    let expected_nonce = match expected_nonce {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing OAuth nonce" })),
            )
                .into_response();
        }
    };

    let token_response = match auth::exchange_code(&state.config, &query.code).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let claims = match auth::decode_id_token(&state.config, &token_response.id_token, Some(&expected_nonce)).await {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let email = match auth::extract_email(&claims) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    if let Err(e) = auth::validate_domain(&email) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    let user = sqlx::query_as!(
        UserRow,
        r#"INSERT INTO users (email) VALUES ($1)
           ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
           RETURNING id, email, role"#,
        email
    )
    .fetch_one(&state.db)
    .await;

    let mut user = match user {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    if state
        .config
        .admin_emails
        .iter()
        .any(|e| e.eq_ignore_ascii_case(&user.email))
        && user.role != "admin"
    {
        let update_result = sqlx::query!(
            "UPDATE users SET role = 'admin' WHERE id = $1 RETURNING role",
            user.id
        )
        .fetch_one(&state.db)
        .await;

        if let Ok(row) = update_result {
            user.role = row.role;
        }
    }

    let session_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(SESSION_DURATION_DAYS);

    let session_result = sqlx::query!(
        r#"INSERT INTO sessions (id, user_id, expires_at) VALUES ($1, $2, $3)"#,
        session_id,
        user.id,
        expires_at
    )
    .execute(&state.db)
    .await;

    if let Err(e) = session_result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    let mut cookie = Cookie::new(SESSION_COOKIE, session_id.to_string());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_max_age(time::Duration::days(SESSION_DURATION_DAYS));
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_secure(secure);

    cookies.add(cookie);

    Redirect::temporary(FRONTEND_URL).into_response()
}

pub async fn me(
    State(state): State<AppState>,
    cookies: Cookies,
) -> (StatusCode, Json<serde_json::Value>) {
    match get_authenticated_user(&state, &cookies).await {
        Ok(u) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": u.id,
                "email": u.email,
                "role": u.role,
            })),
        ),
        Err(e) => e,
    }
}

pub async fn logout(State(state): State<AppState>, cookies: Cookies) -> Redirect {
    if let Some(session_id) = get_session_id(&cookies) {
        let _ = sqlx::query!(r#"DELETE FROM sessions WHERE id = $1"#, session_id)
            .execute(&state.db)
            .await;
    }

    let mut removal = Cookie::from(SESSION_COOKIE);
    removal.set_path("/");
    removal.set_secure(should_use_secure_cookies(&state));
    cookies.remove(removal);

    Redirect::temporary(FRONTEND_URL)
}

pub async fn get_authenticated_user(
    state: &AppState,
    cookies: &Cookies,
) -> Result<UserRow, (StatusCode, Json<serde_json::Value>)> {
    let session_id = match get_session_id(cookies) {
        Some(id) => id,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Not authenticated" })),
            ));
        }
    };

    let user = sqlx::query_as!(
        UserRow,
        r#"SELECT u.id, u.email, u.role
           FROM users u
           JOIN sessions s ON s.user_id = u.id
           WHERE s.id = $1 AND s.expires_at > NOW()"#,
        session_id
    )
    .fetch_optional(&state.db)
    .await;

    match user {
        Ok(Some(u)) => Ok(u),
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Session expired or invalid" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

pub fn get_session_id(cookies: &Cookies) -> Option<Uuid> {
    cookies
        .get(SESSION_COOKIE)
        .and_then(|c| Uuid::parse_str(c.value()).ok())
}

fn should_use_secure_cookies(state: &AppState) -> bool {
    state.config.ms_redirect_uri.starts_with("https://")
}

fn build_cookie(name: &'static str, value: String, secure: bool, max_age: time::Duration) -> Cookie<'static> {
    let mut cookie = Cookie::new(name, value);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_secure(secure);
    cookie.set_max_age(max_age);
    cookie
}

fn clear_cookie(cookies: &Cookies, name: &'static str, secure: bool) {
    let mut removal = Cookie::from(name);
    removal.set_path("/");
    removal.set_secure(secure);
    cookies.remove(removal);
}

#[derive(Debug)]
pub struct UserRow {
    pub id: i32,
    pub email: String,
    pub role: String,
}
