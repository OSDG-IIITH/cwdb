use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ResourceRow {
    pub id: i32,
    pub source_id: i32,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub file_path: String,
    pub title: String,
    pub r#type: String,
    pub like_count: i32,
}

pub async fn list_resources(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as!(
        ResourceRow,
        r#"
        SELECT r.id, r.source_id, s.owner, s.repo, s.branch, r.file_path, r.title, r.type, r.like_count 
        FROM resources r 
        JOIN sources s ON r.source_id = s.id 
        WHERE s.source_status NOT IN ('archived', 'error')
        "#
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => (
            StatusCode::OK,
            Json(serde_json::json!({ "resources": rows })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
