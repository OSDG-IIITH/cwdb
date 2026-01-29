use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub id: i32,
    pub filename: String,
    pub path: String,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub tags: SearchTags,
}

#[derive(Debug, Serialize)]
pub struct SearchTags {
    pub course: Option<String>,
    #[serde(rename = "shortCourse")]
    pub short_course: Option<String>,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct MeiliHit {
    id: i32,
    file_path: String,
    title: String,
    owner: String,
    repo: String,
    branch: String,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.unwrap_or_default();
    let index = state.meili.index("resources");

    match index.search().with_query(&q).execute::<MeiliHit>().await {
        Ok(results) => {
            let hits: Vec<SearchHit> = results
                .hits
                .into_iter()
                .map(|h| {
                    let hit = h.result;
                    let filename = hit
                        .file_path
                        .rsplit('/')
                        .next()
                        .unwrap_or(&hit.file_path)
                        .to_string();

                    let file_type = infer_file_type(&filename);

                    SearchHit {
                        id: hit.id,
                        filename,
                        path: hit.file_path,
                        owner: hit.owner,
                        repo: hit.repo,
                        branch: hit.branch,
                        tags: SearchTags {
                            course: Some(hit.title.clone()),
                            short_course: None,
                            file_type,
                        },
                    }
                })
                .collect();

            (StatusCode::OK, Json(SearchResponse { hits }))
        }
        Err(e) => {
            tracing::error!("Meilisearch error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SearchResponse { hits: vec![] }),
            )
        }
    }
}

fn infer_file_type(filename: &str) -> Option<String> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    match ext.as_str() {
        "pdf" => Some("PDF".to_string()),
        "py" => Some("Python".to_string()),
        "cpp" | "c" | "h" => Some("C/C++".to_string()),
        "pptx" | "ppt" => Some("Slides".to_string()),
        "ipynb" => Some("Notebook".to_string()),
        "md" => Some("Markdown".to_string()),
        "tex" => Some("LaTeX".to_string()),
        _ => None,
    }
}
