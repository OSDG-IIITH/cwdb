use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};


use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestDocument {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateResource {
    pub title: String,
    pub file_path: String,
}

pub async fn index_document(
    State(state): State<AppState>,
    Json(doc): Json<TestDocument>,
) -> impl IntoResponse {
    let index = state.meili.index("resources");

    match index.add_documents(&[doc], Some("id")).await {
        Ok(task) => (StatusCode::ACCEPTED, Json(serde_json::json!({ "taskUid": task.task_uid }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn search_documents(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let index = state.meili.index("resources");

    match index.search().with_query(&query.q).execute::<TestDocument>().await {
        Ok(results) => {
            let hits: Vec<_> = results.hits.into_iter().map(|h| h.result).collect();
            (StatusCode::OK, Json(serde_json::json!({ "hits": hits })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn create_resource(
    State(state): State<AppState>,
    Json(payload): Json<CreateResource>,
) -> impl IntoResponse {
    let result = sqlx::query_as!(
        ResourceRow,
        r#"INSERT INTO resources (title, file_path) VALUES ($1, $2) RETURNING id, title, file_path, created_at"#,
        payload.title,
        payload.file_path
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (StatusCode::CREATED, Json(serde_json::json!(row))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn list_resources(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query_as!(ResourceRow, r#"SELECT id, title, file_path, created_at FROM resources"#)
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(rows) => (StatusCode::OK, Json(serde_json::json!({ "resources": rows }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn sync_resource(
    State(state): State<AppState>,
    Json(payload): Json<CreateResource>,
) -> impl IntoResponse {
    let db_result = sqlx::query_as!(
        ResourceRow,
        r#"INSERT INTO resources (title, file_path) VALUES ($1, $2) RETURNING id, title, file_path, created_at"#,
        payload.title,
        payload.file_path
    )
    .fetch_one(&state.db)
    .await;

    let row = match db_result {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("DB: {}", e) })),
            )
        }
    };

    let doc = TestDocument {
        id: row.id as i64,
        title: row.title.clone(),
        file_path: Some(row.file_path.clone()),
    };

    let index = state.meili.index("resources");
    if let Err(e) = index.add_documents(&[doc], Some("id")).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Meilisearch: {}", e) })),
        );
    }

    (StatusCode::CREATED, Json(serde_json::json!(row)))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ResourceRow {
    pub id: i32,
    pub title: String,
    pub file_path: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
