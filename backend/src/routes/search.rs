use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceHit {
    pub id: i32,
    pub source_id: i32,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub file_path: String,
    pub title: String,
    pub r#type: Option<String>,
    pub like_count: i32,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub resources: Vec<ResourceHit>,
}

#[derive(Debug, Deserialize)]
struct MeiliHit {
    id: i32,
    source_id: i32,
    file_path: String,
    title: String,
    owner: String,
    repo: String,
    branch: String,
    #[serde(rename = "type")]
    resource_type: Option<String>,
    like_count: i32,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.unwrap_or_default();
    let index = state.meili.index("resources");

    match index.search().with_query(&q).execute::<MeiliHit>().await {
        Ok(results) => {
            let resources: Vec<ResourceHit> = results
                .hits
                .into_iter()
                .map(|h| {
                    let hit = h.result;
                    ResourceHit {
                        id: hit.id,
                        source_id: hit.source_id,
                        file_path: hit.file_path,
                        title: hit.title,
                        owner: hit.owner,
                        repo: hit.repo,
                        branch: hit.branch,
                        r#type: hit.resource_type,
                        like_count: hit.like_count,
                    }
                })
                .collect();

            (StatusCode::OK, Json(SearchResponse { resources }))
        }
        Err(e) => {
            tracing::error!("Meilisearch error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SearchResponse { resources: vec![] }),
            )
        }
    }
}
