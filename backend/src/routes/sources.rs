use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{github::GitHubClient, AppState};

#[derive(Debug, Deserialize)]
pub struct CreateSource {
    pub owner: String,
    pub repo: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SourceRow {
    pub id: i32,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub source_status: String,
    pub last_synced_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ResourceRow {
    pub id: i32,
    pub source_id: i32,
    pub file_path: String,
    pub title: String,
    pub vote_count: i32,
}

pub async fn create_source(
    State(state): State<AppState>,
    Json(payload): Json<CreateSource>,
) -> impl IntoResponse {
    let github = GitHubClient::new();

    let branch = match &payload.branch {
        Some(b) => b.clone(),
        None => match github.get_default_branch(&payload.owner, &payload.repo).await {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            }
        },
    };

    let result = sqlx::query_as!(
        SourceRow,
        r#"INSERT INTO sources (owner, repo, branch)
           VALUES ($1, $2, $3)
           ON CONFLICT (owner, repo, branch) DO UPDATE SET owner = EXCLUDED.owner
           RETURNING id, owner, repo, branch, source_status, last_synced_at, created_at"#,
        payload.owner,
        payload.repo,
        branch
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

pub async fn list_sources(State(state): State<AppState>) -> impl IntoResponse {
    let result = sqlx::query_as!(
        SourceRow,
        r#"SELECT id, owner, repo, branch, source_status, last_synced_at, created_at FROM sources"#
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(rows) => (StatusCode::OK, Json(serde_json::json!({ "sources": rows }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn sync_source(
    State(state): State<AppState>,
    Path(source_id): Path<i32>,
) -> impl IntoResponse {
    let source = sqlx::query_as!(
        SourceRow,
        r#"SELECT id, owner, repo, branch, source_status, last_synced_at, created_at FROM sources WHERE id = $1"#,
        source_id
    )
    .fetch_optional(&state.db)
    .await;

    let source = match source {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Source not found" })),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    let github = GitHubClient::new();
    let tree = match github.get_repo_tree(&source.owner, &source.repo, Some(&source.branch)).await {
        Ok((_, t)) => t,
        Err(e) => {
            if matches!(e, crate::github::GitHubError::NotFound) {
                let _ = sqlx::query!(
                    r#"UPDATE sources SET source_status = 'archived' WHERE id = $1"#,
                    source_id
                )
                .execute(&state.db)
                .await;
            }
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    let mut inserted = 0;
    let mut updated = 0;

    for entry in &tree {
        let path_hash = compute_path_hash(&entry.path);
        let title = extract_title(&entry.path);
        let download_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            source.owner, source.repo, source.branch, entry.path
        );

        let result = sqlx::query!(
            r#"INSERT INTO resources (source_id, file_path, path_hash, title, download_url, sha)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (source_id, path_hash) DO UPDATE
               SET file_path = EXCLUDED.file_path,
                   title = EXCLUDED.title,
                   download_url = EXCLUDED.download_url,
                   sha = EXCLUDED.sha,
                   updated_at = NOW()
               RETURNING (xmax = 0) as is_insert"#,
            source_id,
            entry.path,
            path_hash,
            title,
            download_url,
            entry.sha
        )
        .fetch_one(&state.db)
        .await;

        if let Ok(row) = result {
            if row.is_insert.unwrap_or(false) {
                inserted += 1;
            } else {
                updated += 1;
            }
        }
    }

    let _ = sqlx::query!(
        r#"UPDATE sources SET last_synced_at = NOW(), source_status = 'active' WHERE id = $1"#,
        source_id
    )
    .execute(&state.db)
    .await;

    let resources = sqlx::query_as!(
        ResourceRow,
        r#"SELECT id, source_id, file_path, title, vote_count FROM resources WHERE source_id = $1"#,
        source_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let index = state.meili.index("resources");
    let docs: Vec<_> = resources
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "source_id": r.source_id,
                "file_path": r.file_path,
                "title": r.title,
                "vote_count": r.vote_count,
                "owner": source.owner,
                "repo": source.repo
            })
        })
        .collect();

    let _ = index.add_documents(&docs, Some("id")).await;

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "inserted": inserted,
            "updated": updated,
            "total": resources.len()
        })),
    )
}

fn compute_path_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_title(path: &str) -> String {
    path.rsplit('/')
        .next()
        .unwrap_or(path)
        .rsplit('.')
        .last()
        .unwrap_or(path)
        .replace(['_', '-'], " ")
}
