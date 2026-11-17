use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use crate::AppState;
use crate::routes::auth::{Claims, upsertuser};

const FIELDS: &str = "autoterm, examfocus, ug1, ug1since, prefs";

/// most synced prefs, and the longest key or value, an account may store
const MAXPREFS: usize = 16;
const MAXLEN: usize = 64;

/// local storage keys the frontend syncs: theme, mode, course view and panel widths
fn synced(key: &str) -> bool {
    matches!(key, "theme-variant" | "mode-watcher-mode" | "courseview") || key.starts_with("cwdb:width:")
}

fn fail(e: sqlx::Error) -> Response {
    let code = match e {
        sqlx::Error::Database(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, Json(json!({ "error": e.to_string() }))).into_response()
}

/// upserts the user and their profile row, returns the user id
async fn owner(state: &AppState, claims: &Claims) -> Result<Uuid, Response> {
    let user = upsertuser(state, claims).await.map_err(|e| e.into_response())?;
    sqlx::query("INSERT INTO profiles (userid) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(fail)?;
    Ok(user.id)
}

fn settings(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    json!({
        "autoterm": r.get::<bool, _>("autoterm"),
        "examfocus": r.get::<bool, _>("examfocus"),
        "ug1": r.get::<bool, _>("ug1"),
        "ug1since": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("ug1since"),
        "prefs": r.get::<serde_json::Value, _>("prefs"),
    })
}

/// GET /api/profile — toggles, my courses and paper progress
pub async fn get(State(state): State<AppState>, claims: Claims) -> Response {
    let id = match owner(&state, &claims).await {
        Ok(id) => id,
        Err(e) => return e,
    };

    let run = async {
        // ug1 lasts until the first may after it was turned on
        sqlx::query(
            "UPDATE profiles SET ug1 = false, ug1since = NULL WHERE userid = $1 AND ug1 AND NOW() >= \
             make_date(extract(year FROM ug1since AT TIME ZONE 'Asia/Kolkata')::int \
             + (extract(month FROM ug1since AT TIME ZONE 'Asia/Kolkata') >= 5)::int, 5, 1) AT TIME ZONE 'Asia/Kolkata'",
        )
        .bind(id)
        .execute(&state.db)
        .await?;

        let profile = sqlx::query(&format!("SELECT {FIELDS} FROM profiles WHERE userid = $1"))
        .bind(id)
        .fetch_one(&state.db)
        .await?;
        let courses = sqlx::query("SELECT course, added, archived FROM usercourses WHERE userid = $1 ORDER BY added")
            .bind(id)
            .fetch_all(&state.db)
            .await?;
        let progress = sqlx::query("SELECT resource, done FROM progress WHERE userid = $1")
            .bind(id)
            .fetch_all(&state.db)
            .await?;

        let mut out = settings(&profile);
        out["courses"] = courses
            .iter()
            .map(|r| {
                json!({
                    "course": r.get::<String, _>("course"),
                    "added": r.get::<chrono::DateTime<chrono::Utc>, _>("added"),
                    "archived": r.get::<bool, _>("archived"),
                })
            })
            .collect();
        out["progress"] = progress
            .iter()
            .map(|r| json!({ "id": r.get::<i32, _>("resource").to_string(), "done": r.get::<bool, _>("done") }))
            .collect();
        Ok::<_, sqlx::Error>(out)
    };

    match run.await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => fail(e),
    }
}

#[derive(Deserialize)]
pub struct Settings {
    pub autoterm: Option<bool>,
    pub examfocus: Option<bool>,
    pub ug1: Option<bool>,
    pub prefs: Option<HashMap<String, String>>,
}

/// PATCH /api/profile — changes any of the toggles, or replaces the synced prefs
pub async fn edit(State(state): State<AppState>, claims: Claims, Json(s): Json<Settings>) -> Response {
    if s.prefs.as_ref().is_some_and(|p| {
        p.len() > MAXPREFS || p.iter().any(|(k, v)| !synced(k) || k.len() > MAXLEN || v.len() > MAXLEN)
    }) {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "prefs must be known keys with short values" })))
            .into_response();
    }

    let id = match owner(&state, &claims).await {
        Ok(id) => id,
        Err(e) => return e,
    };

    let row = sqlx::query(&format!(
        "UPDATE profiles SET \
         autoterm = COALESCE($2, autoterm), \
         examfocus = COALESCE($3, examfocus), \
         ug1since = CASE WHEN $4 AND NOT ug1 THEN NOW() WHEN NOT $4 THEN NULL ELSE ug1since END, \
         ug1 = COALESCE($4, ug1), \
         prefs = COALESCE($5, prefs) \
         WHERE userid = $1 RETURNING {FIELDS}"
    ))
    .bind(id)
    .bind(s.autoterm)
    .bind(s.examfocus)
    .bind(s.ug1)
    .bind(s.prefs.map(|p| json!(p)))
    .fetch_one(&state.db)
    .await;

    match row {
        Ok(r) => (StatusCode::OK, Json(settings(&r))).into_response(),
        Err(e) => fail(e),
    }
}

#[derive(Deserialize)]
pub struct Courses {
    pub courses: Vec<String>,
    pub archived: Option<bool>,
}

/// POST /api/profile/courses — adds courses, or brings archived ones back as newly added; unknown codes are skipped
pub async fn add(State(state): State<AppState>, claims: Claims, Json(c): Json<Courses>) -> Response {
    let id = match owner(&state, &claims).await {
        Ok(id) => id,
        Err(e) => return e,
    };

    let rows = sqlx::query(
        "INSERT INTO usercourses (userid, course) SELECT $1, code FROM courses WHERE code = ANY($2) \
         ON CONFLICT (userid, course) DO UPDATE SET added = NOW(), archived = false \
         RETURNING course, added, archived",
    )
    .bind(id)
    .bind(&c.courses)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let out: Vec<_> = rows
                .iter()
                .map(|r| {
                    json!({
                        "course": r.get::<String, _>("course"),
                        "added": r.get::<chrono::DateTime<chrono::Utc>, _>("added"),
                        "archived": r.get::<bool, _>("archived"),
                    })
                })
                .collect();
            (StatusCode::CREATED, Json(json!({ "courses": out }))).into_response()
        }
        Err(e) => fail(e),
    }
}

/// PATCH /api/profile/courses — archives or unarchives courses by hand
pub async fn archive(State(state): State<AppState>, claims: Claims, Json(c): Json<Courses>) -> Response {
    let id = match owner(&state, &claims).await {
        Ok(id) => id,
        Err(e) => return e,
    };
    let Some(archived) = c.archived else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "archived is required" }))).into_response();
    };

    match sqlx::query("UPDATE usercourses SET archived = $3 WHERE userid = $1 AND course = ANY($2)")
        .bind(id)
        .bind(&c.courses)
        .bind(archived)
        .execute(&state.db)
        .await
    {
        Ok(r) => (StatusCode::OK, Json(json!({ "updated": r.rows_affected() }))).into_response(),
        Err(e) => fail(e),
    }
}

/// DELETE /api/profile/courses/{code}
pub async fn remove(State(state): State<AppState>, claims: Claims, Path(code): Path<String>) -> Response {
    let id = match owner(&state, &claims).await {
        Ok(id) => id,
        Err(e) => return e,
    };

    match sqlx::query("DELETE FROM usercourses WHERE userid = $1 AND course = $2")
        .bind(id)
        .bind(code)
        .execute(&state.db)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => fail(e),
    }
}

#[derive(Deserialize)]
pub struct Mark {
    pub done: Option<bool>,
}

/// PUT /api/profile/progress/{id} — records a paper as seen, and optionally sets it done
pub async fn mark(
    State(state): State<AppState>,
    claims: Claims,
    Path(resource): Path<i32>,
    Json(m): Json<Mark>,
) -> Response {
    let id = match owner(&state, &claims).await {
        Ok(id) => id,
        Err(e) => return e,
    };

    let row = sqlx::query(
        "INSERT INTO progress (userid, resource, done) VALUES ($1, $2, COALESCE($3, false)) \
         ON CONFLICT (userid, resource) DO UPDATE SET seen = NOW(), done = COALESCE($3, progress.done) \
         RETURNING done",
    )
    .bind(id)
    .bind(resource)
    .bind(m.done)
    .fetch_one(&state.db)
    .await;

    match row {
        Ok(r) => (StatusCode::OK, Json(json!({ "id": resource.to_string(), "done": r.get::<bool, _>("done") }))).into_response(),
        Err(e) => fail(e),
    }
}
