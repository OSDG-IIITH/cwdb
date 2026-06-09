use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;

use crate::AppState;
use crate::routes::auth::{Claims, upsertuser};

pub async fn publish(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    if user.role != "admin" {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "admin only"}))).into_response();
    }

    let source_rows = sqlx::query(
        "SELECT id, owner, repo, branch FROM sources WHERE source_status NOT IN ('archived', 'error')"
    )
    .fetch_all(&state.db)
    .await;

    let source_rows = match source_rows {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let resource_rows = sqlx::query(
        "SELECT r.id, r.source_id, r.file_path, r.type as rtype, r.course_id \
         FROM resources r \
         JOIN sources s ON r.source_id = s.id \
         WHERE s.source_status NOT IN ('archived', 'error')"
    )
    .fetch_all(&state.db)
    .await;

    let resource_rows = match resource_rows {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let course_rows = sqlx::query(
        "SELECT id, code, name, aliases, instructors, semester, year FROM courses"
    )
    .fetch_all(&state.db)
    .await;

    let course_rows = match course_rows {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let version = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut sources: HashMap<String, serde_json::Value> = HashMap::new();
    for row in &source_rows {
        let id: i32 = row.get("id");
        let owner: String = row.get("owner");
        let repo: String = row.get("repo");
        let branch: String = row.get("branch");
        sources.insert(id.to_string(), json!({
            "name": format!("{}/{}", owner, repo),
            "url": format!("https://github.com/{}/{}", owner, repo),
            "owner": owner,
            "repo": repo,
            "branch": branch,
        }));
    }

    let mut courses: HashMap<String, serde_json::Value> = HashMap::new();
    for row in &course_rows {
        let id: uuid::Uuid = row.get("id");
        let code: String = row.get("code");
        let name: String = row.get("name");
        let aliases: Vec<String> = row.get("aliases");
        let instructors: Vec<String> = row.get("instructors");
        let semester: String = row.get("semester");
        let year: Option<i32> = row.get("year");
        courses.insert(id.to_string(), json!({
            "name": name,
            "code": code,
            "aliases": aliases,
            "instructors": instructors,
            "semester": semester,
            "year": year,
        }));
    }

    let mut resources = Vec::new();
    for row in &resource_rows {
        let id: i32 = row.get("id");
        let source_id: i32 = row.get("source_id");
        let file_path: String = row.get("file_path");
        let rtype: String = row.get("rtype");
        let course_id: Option<uuid::Uuid> = row.get("course_id");
        resources.push(json!({
            "id": id.to_string(),
            "path": file_path,
            "source_id": source_id.to_string(),
            "course_id": course_id.map(|c| c.to_string()),
            "type": if rtype.is_empty() { json!(null) } else { json!(rtype) },
        }));
    }

    let index = json!({
        "version": version,
        "resources": resources,
        "courses": courses,
        "sources": sources,
    });

    let path = std::path::Path::new(&state.config.publish_path).join("index.json");
    if let Some(dir) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("mkdir: {}", e)}))).into_response();
        }
    }

    match std::fs::write(&path, index.to_string()) {
        Ok(_) => (StatusCode::OK, Json(json!({
            "version": version,
            "resources": resources.len(),
            "courses": courses.len(),
            "sources": sources.len(),
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("write: {}", e)}))).into_response(),
    }
}
