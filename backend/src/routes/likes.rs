use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::AppState;

const SESSION_COOKIE: &str = "cwdb_session";

async fn get_user_id_from_session(state: &AppState, cookies: &Cookies) -> Option<i32> {
    let session_id = cookies
        .get(SESSION_COOKIE)
        .and_then(|c| Uuid::parse_str(c.value()).ok())?;

    sqlx::query_scalar!(
        r#"SELECT u.id FROM users u
           JOIN sessions s ON s.user_id = u.id
           WHERE s.id = $1 AND s.expires_at > NOW()"#,
        session_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
}

pub async fn toggle_like(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(resource_id): Path<i32>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_session(&state, &cookies).await {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Authentication required" })),
            )
        }
    };

    let existing = sqlx::query!(
        r#"SELECT 1 as exists FROM likes WHERE user_id = $1 AND resource_id = $2"#,
        user_id,
        resource_id
    )
    .fetch_optional(&state.db)
    .await;

    let liked: bool;

    match existing {
        Ok(Some(_)) => {
            let _ = sqlx::query!(
                r#"DELETE FROM likes WHERE user_id = $1 AND resource_id = $2"#,
                user_id,
                resource_id
            )
            .execute(&state.db)
            .await;

            let _ = sqlx::query!(
                r#"UPDATE resources SET like_count = like_count - 1 WHERE id = $1"#,
                resource_id
            )
            .execute(&state.db)
            .await;

            liked = false;
        }
        Ok(None) => {
            let insert = sqlx::query!(
                r#"INSERT INTO likes (user_id, resource_id) VALUES ($1, $2)"#,
                user_id,
                resource_id
            )
            .execute(&state.db)
            .await;

            if insert.is_err() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Resource not found" })),
                );
            }

            let _ = sqlx::query!(
                r#"UPDATE resources SET like_count = like_count + 1 WHERE id = $1"#,
                resource_id
            )
            .execute(&state.db)
            .await;

            liked = true;
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    }

    let like_count = sqlx::query_scalar!(
        r#"SELECT like_count FROM resources WHERE id = $1"#,
        resource_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    (
        StatusCode::OK,
        Json(serde_json::json!({ "liked": liked, "like_count": like_count })),
    )
}

pub async fn get_likes(
    State(state): State<AppState>,
    Path(resource_id): Path<i32>,
) -> impl IntoResponse {
    let like_count = sqlx::query_scalar!(
        r#"SELECT like_count FROM resources WHERE id = $1"#,
        resource_id
    )
    .fetch_optional(&state.db)
    .await;

    match like_count {
        Ok(Some(count)) => (StatusCode::OK, Json(serde_json::json!({ "like_count": count }))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Resource not found" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}


pub async fn toggle_source_like(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(source_id): Path<i32>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_session(&state, &cookies).await {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Authentication required" })),
            )
        }
    };

    let existing = sqlx::query!(
        r#"SELECT 1 as exists FROM source_likes WHERE user_id = $1 AND source_id = $2"#,
        user_id,
        source_id
    )
    .fetch_optional(&state.db)
    .await;

    let liked: bool;

    match existing {
        Ok(Some(_)) => {
            let _ = sqlx::query!(
                r#"DELETE FROM source_likes WHERE user_id = $1 AND source_id = $2"#,
                user_id,
                source_id
            )
            .execute(&state.db)
            .await;

            let _ = sqlx::query!(
                r#"UPDATE sources SET like_count = like_count - 1 WHERE id = $1"#,
                source_id
            )
            .execute(&state.db)
            .await;

            liked = false;
        }
        Ok(None) => {
            let insert = sqlx::query!(
                r#"INSERT INTO source_likes (user_id, source_id) VALUES ($1, $2)"#,
                user_id,
                source_id
            )
            .execute(&state.db)
            .await;

            if insert.is_err() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Source not found" })),
                );
            }

            let _ = sqlx::query!(
                r#"UPDATE sources SET like_count = like_count + 1 WHERE id = $1"#,
                source_id
            )
            .execute(&state.db)
            .await;

            liked = true;
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    }

    let like_count = sqlx::query_scalar!(
        r#"SELECT like_count FROM sources WHERE id = $1"#,
        source_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    (
        StatusCode::OK,
        Json(serde_json::json!({ "liked": liked, "like_count": like_count })),
    )
}
