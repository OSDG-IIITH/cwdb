use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tower_cookies::Cookies;

use crate::{github::GitHubClient, routes::auth::get_authenticated_user, AppState};

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
    pub created_by: i32,
    pub like_count: i32,
    pub liked: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListSourcesQuery {
    pub filter: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ResourceRow {
    pub id: i32,
    pub source_id: i32,
    pub file_path: String,
    pub title: String,
    pub r#type: String,
    pub like_count: i32,
}

pub async fn create_source(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<CreateSource>,
) -> impl IntoResponse {
    let user = match get_authenticated_user(&state, &cookies).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

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
                    .into_response()
            }
        },
    };

    let result = sqlx::query_as!(
        SourceRow,
        r#"INSERT INTO sources (owner, repo, branch, created_by)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (owner, repo, branch) DO UPDATE 
           SET owner = EXCLUDED.owner, created_by = EXCLUDED.created_by
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
    cookies: Cookies,
    Query(query): Query<ListSourcesQuery>,
) -> impl IntoResponse {
    let current_user_id = match get_authenticated_user(&state, &cookies).await {
        Ok(u) => Some(u.id),
        Err(_) => None,
    };

    let filter_user_id = if let Some(filter) = &query.filter {
        if filter == "mine" {
            if let Some(uid) = current_user_id {
                Some(uid)
            } else {
                 return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "Authentication required for filter=mine" }))).into_response();
            }
        } else {
            None
        }
    } else {
        None
    };

    let result = if let Some(uid) = filter_user_id {
        sqlx::query_as!(
            SourceRow,
            r#"SELECT 
                s.id, s.owner, s.repo, s.branch, s.source_status, s.last_synced_at, s.created_at, s.created_by, s.like_count,
                EXISTS(SELECT 1 FROM source_likes sl WHERE sl.source_id = s.id AND sl.user_id = $1) as liked
               FROM sources s 
               WHERE s.created_by = $1"#,
            uid
        )
        .fetch_all(&state.db)
        .await
    } else {
        match current_user_id {
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
    cookies: Cookies,
    Path(source_id): Path<i32>,
) -> impl IntoResponse {
    let user = match get_authenticated_user(&state, &cookies).await {
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
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if user.role != "admin" && source.created_by != user.id {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "You do not have permission to sync this source" })),
        )
            .into_response();
    }

    let settings = sqlx::query!(
        "SELECT value FROM settings WHERE key = 'global_ignore_patterns'"
    )
    .fetch_optional(&state.db)
    .await;

    let ignore_patterns: Vec<String> = match settings {
        Ok(Some(row)) => serde_json::from_value(row.value).unwrap_or_default(),
        _ => vec![],
    };

    let github = GitHubClient::new();
    let tree = match github.get_repo_tree(&source.owner, &source.repo, Some(&source.branch), &ignore_patterns).await {
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

        let result = sqlx::query!(
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
        r#"SELECT id, source_id, file_path, title, type, like_count FROM resources WHERE source_id = $1"#,
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
                "like_count": r.like_count,
                "owner": source.owner,
                "repo": source.repo,
                "branch": source.branch,
                "type": r.r#type
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
        .into_response()
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
    let filename = path.split('/').last().unwrap_or(path).to_lowercase();
    if filename.contains("end") || filename.contains("mid") || filename.contains("quiz") {
        return "exam".to_string();
    }
    if filename.contains("lecture") {
        return "slides".to_string();
    }
    "".to_string()
}
