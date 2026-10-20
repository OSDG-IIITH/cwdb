use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;

use crate::AppState;
use crate::routes::auth::{Claims, adminonly, upsertuser};

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

    let started = chrono::Utc::now();

    let resource_rows = sqlx::query(
        "SELECT r.id, r.source_id, r.file_path, r.course, r.offering, r.kind, r.examtype, r.slidetype, \
         r.n, r.key, r.keyfor, r.title, r.unit, r.size, r.needsreview \
         FROM resources r \
         JOIN sources s ON r.source_id = s.id \
         WHERE s.source_status NOT IN ('archived', 'error') AND NOT r.hidden"
    )
    .fetch_all(&state.db)
    .await;

    let resource_rows = match resource_rows {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let course_rows = sqlx::query("SELECT code, name, shortnames, semester FROM courses")
        .fetch_all(&state.db)
        .await;

    let course_rows = match course_rows {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let offering_rows = sqlx::query(
        "SELECT o.id, o.course, o.season, o.year, array_remove(array_agg(f.name ORDER BY f.name), NULL) AS faculty \
         FROM offerings o LEFT JOIN offeringfaculty f ON f.offering = o.id GROUP BY o.id"
    )
    .fetch_all(&state.db)
    .await;

    let offering_rows = match offering_rows {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let succession = sqlx::query_as::<_, (String, String)>("SELECT predecessor, successor FROM coursesuccession")
        .fetch_all(&state.db)
        .await;

    let succession = match succession {
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
        let code: String = row.get("code");
        let name: String = row.get("name");
        let shortnames: Vec<String> = row.get("shortnames");
        let semester: String = row.get("semester");
        courses.insert(code.clone(), json!({
            "name": name,
            "code": code,
            "shortnames": shortnames,
            "aliases": shortnames,
            "semester": semester,
        }));
    }

    let mut offerings: HashMap<String, serde_json::Value> = HashMap::new();
    let mut faculty: HashMap<String, Vec<String>> = HashMap::new();
    for row in &offering_rows {
        let id: i32 = row.get("id");
        let course: String = row.get("course");
        let season: String = row.get("season");
        let year: i16 = row.get("year");
        let names: Vec<String> = row.get("faculty");
        offerings.insert(id.to_string(), json!({ "course": course, "season": season, "year": year }));
        if !names.is_empty() {
            faculty.insert(id.to_string(), names);
        }
    }

    let mut resources = Vec::new();
    for row in &resource_rows {
        let id: i32 = row.get("id");
        let source_id: i32 = row.get("source_id");
        let file_path: String = row.get("file_path");
        let course: Option<String> = row.get("course");
        let kind: Option<String> = row.get("kind");
        let legacy = match kind.as_deref() {
            Some("exams") => json!("exam"),
            Some("slides") => json!("slides"),
            _ => json!(null),
        };
        let mut r = json!({
            "id": id.to_string(),
            "path": file_path,
            "source_id": source_id.to_string(),
            "course_id": course,
            "type": legacy,
        });
        let fields = [
            ("kind", json!(kind)),
            ("examtype", json!(row.get::<Option<String>, _>("examtype"))),
            ("slidetype", json!(row.get::<Option<String>, _>("slidetype"))),
            ("n", json!(row.get::<Option<i32>, _>("n"))),
            ("offering", json!(row.get::<Option<i32>, _>("offering").map(|o| o.to_string()))),
            ("keyfor", json!(row.get::<Option<i32>, _>("keyfor").map(|k| k.to_string()))),
            ("title", json!(row.get::<Option<String>, _>("title"))),
            ("unit", json!(row.get::<Option<String>, _>("unit"))),
            ("size", json!(row.get::<Option<i64>, _>("size"))),
            ("key", json!(row.get::<bool, _>("key"))),
            ("review", json!(row.get::<bool, _>("needsreview"))),
        ];
        for (k, v) in fields {
            if !v.is_null() && v != json!(false) {
                r[k] = v;
            }
        }
        resources.push(r);
    }

    let index = json!({
        "version": version,
        "resources": resources,
        "courses": courses,
        "sources": sources,
        "offerings": offerings,
        "faculty": faculty,
        "succession": succession,
    });

    let path = std::path::Path::new(&state.config.publish_path).join("index.json");
    if let Some(dir) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("mkdir: {}", e)}))).into_response();
        }
    }

    if let Err(e) = std::fs::write(&path, index.to_string()) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("write: {}", e)}))).into_response();
    }

    let saved = sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('lastpublish', to_jsonb($1::timestamptz)) \
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"
    )
    .bind(started)
    .execute(&state.db)
    .await;

    match saved {
        Ok(_) => (StatusCode::OK, Json(json!({
            "version": version,
            "resources": resources.len(),
            "courses": courses.len(),
            "sources": sources.len(),
            "offerings": offerings.len(),
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/publish/pending — resources changed since the last publish
pub async fn pending(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    if let Err(e) = adminonly(&state, &claims).await {
        return e;
    }

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM resources WHERE updated_at > COALESCE( \
         (SELECT (value #>> '{}')::timestamptz FROM settings WHERE key = 'lastpublish'), '-infinity')"
    )
    .fetch_one(&state.db)
    .await;

    match count {
        Ok(n) => (StatusCode::OK, Json(json!({ "pending": n }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
