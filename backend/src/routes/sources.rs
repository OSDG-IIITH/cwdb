use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::time::{Duration, sleep};

use crate::AppState;
use crate::routes::auth::{OptionalAuth, upsertuser};
use ocas_auth::Claims;

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
    pub created_by: uuid::Uuid,
    pub like_count: i32,
    pub liked: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ResourceRow {
    pub id: i32,
    pub source_id: i32,
    pub file_path: String,
    pub title: String,
    pub r#type: String,
    pub like_count: i32,
    pub course_id: Option<uuid::Uuid>,
}

pub async fn create_source(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateSource>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    if user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only admins can add sources" })),
        )
            .into_response();
    }

    let github = crate::github::GitHubClient::new();

    let branch = match &payload.branch {
        Some(b) => b.clone(),
        None => match github
            .get_default_branch(&payload.owner, &payload.repo)
            .await
        {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response();
            }
        },
    };

    let result = sqlx::query_as!(
        SourceRow,
        r#"INSERT INTO sources (owner, repo, branch, created_by)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (owner, repo, branch) DO UPDATE 
           SET owner = EXCLUDED.owner
           RETURNING id, owner, repo, branch, source_status, last_synced_at, created_at, created_by, like_count, false as liked"#,
        payload.owner,
        payload.repo,
        branch,
        user.id
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (StatusCode::CREATED, Json(serde_json::json!(row))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn list_sources(
    State(state): State<AppState>,
    auth: OptionalAuth,
) -> impl IntoResponse {
    let current_user_id = match auth {
        OptionalAuth::Authenticated(u) => Some(u.id),
        OptionalAuth::Anonymous => None,
    };

    let result = match current_user_id {
        Some(uid) => {
             sqlx::query_as!(
                SourceRow,
                r#"SELECT 
                    s.id, s.owner, s.repo, s.branch, s.source_status, s.last_synced_at, s.created_at, s.created_by, s.like_count,
                    EXISTS(SELECT 1 FROM source_likes sl WHERE sl.source_id = s.id AND sl.user_id = $1) as liked
                   FROM sources s"#,
                uid
            )
            .fetch_all(&state.db)
            .await
        },
        None => {
             sqlx::query_as!(
                SourceRow,
                r#"SELECT 
                    s.id, s.owner, s.repo, s.branch, s.source_status, s.last_synced_at, s.created_at, s.created_by, s.like_count,
                    false as liked
                   FROM sources s"#
            )
            .fetch_all(&state.db)
            .await
        }
    };

    match result {
        Ok(rows) => (StatusCode::OK, Json(serde_json::json!({ "sources": rows }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn sync_source(
    State(state): State<AppState>,
    claims: Claims,
    Path(source_id): Path<i32>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    let source = sqlx::query_as!(
        SourceRow,
        r#"SELECT 
            s.id, s.owner, s.repo, s.branch, s.source_status, s.last_synced_at, s.created_at, s.created_by, s.like_count, 
            EXISTS(SELECT 1 FROM source_likes sl WHERE sl.source_id = s.id AND sl.user_id = $2) as liked
           FROM sources s WHERE s.id = $1"#,
        source_id,
        user.id
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

    if user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Only admins can sync sources" })),
        )
            .into_response();
    }

    let settings = sqlx::query!("SELECT value FROM settings WHERE key = 'global_ignore_patterns'")
        .fetch_optional(&state.db)
        .await;

    let ignore_patterns: Vec<String> = match settings {
        Ok(Some(row)) => serde_json::from_value(row.value).unwrap_or_default(),
        _ => vec![],
    };

    let github = crate::github::GitHubClient::new();
    let tree = match github
        .get_repo_tree(
            &source.owner,
            &source.repo,
            Some(&source.branch),
            &ignore_patterns,
        )
        .await
    {
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
            )
                .into_response();
        }
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("failed to start transaction: {}", e) })),
            )
                .into_response();
        }
    };

    let mut inserted = 0;
    let mut updated = 0;

    for entry in &tree {
        let path_hash = compute_path_hash(&entry.path);
        let title = extract_title(&entry.path);
        let resource_type = determine_resource_type(&entry.path);
        let download_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            source.owner, source.repo, source.branch, entry.path
        );

        let row = match sqlx::query!(
            r#"INSERT INTO resources (source_id, file_path, path_hash, title, download_url, type, sha)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (source_id, path_hash) DO UPDATE
               SET file_path = EXCLUDED.file_path,
                   title = EXCLUDED.title,
                   download_url = EXCLUDED.download_url,
                   type = EXCLUDED.type,
                   sha = EXCLUDED.sha,
                   updated_at = NOW()
               RETURNING (xmax = 0) as is_insert"#,
            source_id,
            entry.path,
            path_hash,
            title,
            download_url,
            resource_type,
            entry.sha
        )
        .fetch_one(&mut *tx)
        .await
        {
            Ok(row) => row,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("failed to upsert resources: {}", e) })),
                )
                    .into_response();
            }
        };

        if row.is_insert.unwrap_or(false) {
            inserted += 1;
            if let Some(cid) = state.courses.resolve(&entry.path, &source.repo) {
                let _ = sqlx::query!(
                    "UPDATE resources SET course_id = $1 WHERE source_id = $2 AND path_hash = $3",
                    cid,
                    source_id,
                    path_hash
                )
                .execute(&mut *tx)
                .await;
            }
        } else {
            updated += 1;
        }
    }

    let current_hashes: Vec<String> = tree
        .iter()
        .map(|entry| compute_path_hash(&entry.path))
        .collect();

    let stale_resource_ids = match sqlx::query_scalar!(
        r#"DELETE FROM resources
           WHERE source_id = $1
             AND NOT (path_hash = ANY($2))
           RETURNING id"#,
        source_id,
        &current_hashes
    )
    .fetch_all(&mut *tx)
    .await
    {
        Ok(ids) => ids,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("failed to remove stale resources: {}", e) })),
            )
                .into_response();
        }
    };

    let resources = match sqlx::query_as!(
        ResourceRow,
        r#"SELECT id, source_id, file_path, title, type, like_count, course_id FROM resources WHERE source_id = $1"#,
        source_id
    )
    .fetch_all(&mut *tx)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("failed to load synced resources: {}", e) })),
            )
                .into_response();
        }
    };

    if let Err(e) = sync_meili_index(&state, &source, &resources, &stale_resource_ids).await {
        return (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": "search index sync failed, rolling back",
                "details": e
            })),
        )
            .into_response();
    }

    if let Err(e) = sqlx::query!(
        r#"UPDATE sources SET last_synced_at = NOW(), source_status = 'active' WHERE id = $1"#,
        source_id
    )
    .execute(&mut *tx)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("failed to update source status: {}", e) })),
        )
            .into_response();
    }

    if let Err(e) = tx.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("failed to commit transaction: {}", e) })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "inserted": inserted,
            "updated": updated,
            "total": resources.len()
        })),
    )
        .into_response()
}

async fn sync_meili_index(
    state: &AppState,
    source: &SourceRow,
    resources: &[ResourceRow],
    stale_resource_ids: &[i32],
) -> Result<(), String> {
    let index = state.meili.index("resources");

    let docs: Vec<_> = resources
        .iter()
        .map(|r| {
            let course_name = r
                .course_id
                .and_then(|cid| state.courses.name(cid))
                .map(String::from);
            serde_json::json!({
                "id": r.id,
                "source_id": r.source_id,
                "file_path": r.file_path,
                "title": r.title,
                "like_count": r.like_count,
                "owner": source.owner,
                "repo": source.repo,
                "branch": source.branch,
                "type": r.r#type,
                "course_id": r.course_id,
                "course_name": course_name
            })
        })
        .collect();

    const MAX_ATTEMPTS: u8 = 3;
    let mut last_error = String::from("unknown meilisearch sync error");

    for attempt in 1..=MAX_ATTEMPTS {
        match sync_meili_index_once(state, &index, &docs, stale_resource_ids).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = e;
                if attempt < MAX_ATTEMPTS {
                    let backoff_ms = 250_u64 * u64::from(attempt);
                    sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }
    }

    Err(last_error)
}

async fn sync_meili_index_once(
    state: &AppState,
    index: &meilisearch_sdk::indexes::Index,
    docs: &[serde_json::Value],
    stale_resource_ids: &[i32],
) -> Result<(), String> {
    for stale_id in stale_resource_ids {
        let task = index
            .delete_document(*stale_id)
            .await
            .map_err(|e| format!("failed to delete stale document {}: {}", stale_id, e))?;

        state
            .meili
            .wait_for_task(task, None, None)
            .await
            .map_err(|e| format!("delete task failed for stale document {}: {}", stale_id, e))?;
    }

    let task = index
        .add_documents(docs, Some("id"))
        .await
        .map_err(|e| format!("failed to add/update documents: {}", e))?;

    state
        .meili
        .wait_for_task(task, None, None)
        .await
        .map_err(|e| format!("add/update task failed: {}", e))?;

    Ok(())
}

fn compute_path_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_title(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn determine_resource_type(path: &str) -> String {
    let filename = path.split('/').next_back().unwrap_or(path).to_lowercase();
    if filename.contains("end") || filename.contains("mid") || filename.contains("quiz") {
        return "exam".to_string();
    }
    if filename.contains("lecture") {
        return "slides".to_string();
    }
    "".to_string()
}

pub async fn delete_source(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    if user.role != "admin" {
        return (StatusCode::FORBIDDEN).into_response();
    }

    let resource_ids = match sqlx::query_scalar!(
        "SELECT id FROM resources WHERE source_id = $1",
        id
    )
    .fetch_all(&state.db)
    .await {
        Ok(ids) => ids,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    let index = state.meili.index("resources");
    for rid in resource_ids {
        let _ = index.delete_document(rid).await;
    }

    match sqlx::query!("DELETE FROM sources WHERE id = $1", id)
        .execute(&state.db)
        .await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
