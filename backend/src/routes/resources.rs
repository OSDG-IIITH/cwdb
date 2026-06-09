use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::AppState;
use crate::routes::auth::{Claims, upsertuser};

pub async fn delete_resource(
    State(state): State<AppState>,
    claims: Claims,
    Path(resource_id): Path<i32>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    if user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only admins can delete resources" })),
        )
            .into_response();
    }

    let exists: Result<Option<sqlx::postgres::PgRow>, _> = sqlx::query("SELECT id FROM resources WHERE id = $1")
        .bind(resource_id)
        .fetch_optional(&state.db)
        .await;

    match exists {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Resource not found" })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
        Ok(Some(_)) => {}
    }

    match sqlx::query!("DELETE FROM resources WHERE id = $1", resource_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Resource deleted" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
