use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use crate::{AppState, routes::auth::get_authenticated_user};

#[derive(Debug, Deserialize)]
pub struct ResourceQuery {
    pub owner: Option<String>,
    pub repo: Option<String>,
}

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
    Query(query): Query<ResourceQuery>,
) -> impl IntoResponse {
    let result = if let (Some(owner), Some(repo)) = (query.owner, query.repo) {
        sqlx::query_as!(
            ResourceRow,
            r#"
            SELECT r.id, r.source_id, s.owner, s.repo, s.branch, r.file_path, r.title, r.type, r.like_count 
            FROM resources r 
            JOIN sources s ON r.source_id = s.id 
            WHERE s.source_status NOT IN ('archived', 'error') AND s.owner = $1 AND s.repo = $2
            "#,
            owner, repo
        )
        .fetch_all(&state.db)
        .await
    } else {
        Ok(vec![])
    };

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

pub async fn delete_resource(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(resource_id): Path<i32>,
) -> impl IntoResponse {
    let user = match get_authenticated_user(&state, &cookies).await {
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

    let resource = sqlx::query!(
        "SELECT file_path, source_id FROM resources WHERE id = $1",
        resource_id
    )
    .fetch_optional(&state.db)
    .await;

    let (file_path, _source_id) = match resource {
        Ok(Some(row)) => (row.file_path, row.source_id),
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
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    if let Err(e) = sqlx::query!("DELETE FROM resources WHERE id = $1", resource_id)
        .execute(&mut *tx)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    let settings = sqlx::query!("SELECT value FROM settings WHERE key = 'global_ignore_patterns'")
        .fetch_optional(&mut *tx)
        .await;

    let mut patterns: Vec<String> = match settings {
        Ok(Some(row)) => serde_json::from_value(row.value).unwrap_or_default(),
        _ => vec![],
    };

    if !patterns.contains(&file_path) {
        patterns.push(file_path);
        let new_value = serde_json::to_value(patterns).unwrap();

        if let Err(e) = sqlx::query!(
            "INSERT INTO settings (key, value) VALUES ('global_ignore_patterns', $1) 
             ON CONFLICT (key) DO UPDATE SET value = $1",
            new_value
        )
        .execute(&mut *tx)
        .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    }

    if let Err(e) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response();
    }

    let index = state.meili.index("resources");
    let _ = index.delete_document(resource_id).await;

    (
        StatusCode::OK,
        Json(serde_json::json!({ "message": "Resource deleted and ignored" })),
    )
        .into_response()
}
