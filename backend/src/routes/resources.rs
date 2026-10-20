use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Postgres, QueryBuilder};
use std::collections::HashMap;

use crate::AppState;
use crate::classify;
use crate::routes::auth::{Claims, adminonly, upsertuser};

pub async fn delete_resource(
    State(state): State<AppState>,
    claims: Claims,
    Path(resource_id): Path<i32>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
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

    let exists: Result<Option<sqlx::postgres::PgRow>, _> = sqlx::query("SELECT id FROM resources WHERE id = $1")
        .bind(resource_id)
        .fetch_optional(&state.db)
        .await;

    match exists {
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
        Ok(Some(_)) => {}
    }

    match sqlx::query!("DELETE FROM resources WHERE id = $1", resource_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "message": "Resource deleted" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

fn nullable<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    serde::Deserialize::deserialize(d).map(Some)
}

/// editable fields; a missing field is left alone, null clears it
#[derive(Deserialize)]
pub struct Edit {
    #[serde(default)]
    pub ids: Vec<i32>,
    #[serde(default, deserialize_with = "nullable")]
    pub course: Option<Option<String>>,
    #[serde(default, deserialize_with = "nullable")]
    pub offering: Option<Option<i32>>,
    #[serde(default, deserialize_with = "nullable")]
    pub kind: Option<Option<String>>,
    #[serde(default, deserialize_with = "nullable")]
    pub examtype: Option<Option<String>>,
    #[serde(default, deserialize_with = "nullable")]
    pub slidetype: Option<Option<String>>,
    #[serde(default, deserialize_with = "nullable")]
    pub n: Option<Option<i32>>,
    #[serde(default, deserialize_with = "nullable")]
    pub keyfor: Option<Option<i32>>,
    #[serde(default, deserialize_with = "nullable")]
    pub title: Option<Option<String>>,
    pub key: Option<bool>,
    pub hidden: Option<bool>,
    pub reset: Option<bool>,
}

fn set<'a, T>(q: &mut QueryBuilder<'a, Postgres>, column: &str, value: Option<T>)
where
    T: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send,
{
    if let Some(v) = value {
        q.push(format!(", {column} = ")).push_bind(v);
    }
}

/// applies an edit to rows and locks them as manual
async fn save(state: &AppState, ids: Vec<i32>, mut edit: Edit) -> axum::response::Response {
    if edit.reset == Some(true) {
        let run = async {
            sqlx::query(
                "UPDATE resources SET classifiedby = NULL, needsreview = false, confidence = NULL, keyfor = NULL, updated_at = NOW() \
                 WHERE id = ANY($1)",
            )
            .bind(&ids)
            .execute(&state.db)
            .await?;
            rerun(state, Some(&ids)).await
        };
        return match run.await {
            Ok(n) => (StatusCode::OK, Json(json!({ "updated": n }))).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
        };
    }
    if let Some(kind) = &edit.kind {
        if kind.as_deref() != Some("exams") {
            edit.examtype = Some(None);
        }
        if kind.as_deref() != Some("slides") {
            edit.slidetype = Some(None);
        }
    }
    let mut q = QueryBuilder::new(
        "UPDATE resources SET classifiedby = 'manual', needsreview = false, classifiedat = NOW(), updated_at = NOW()",
    );
    set(&mut q, "course", edit.course);
    set(&mut q, "offering", edit.offering);
    set(&mut q, "kind", edit.kind);
    set(&mut q, "examtype", edit.examtype);
    set(&mut q, "slidetype", edit.slidetype);
    set(&mut q, "n", edit.n);
    set(&mut q, "keyfor", edit.keyfor);
    set(&mut q, "title", edit.title);
    set(&mut q, "key", edit.key);
    set(&mut q, "hidden", edit.hidden);
    q.push(" WHERE id = ANY(").push_bind(ids).push(")");

    match q.build().execute(&state.db).await {
        Ok(r) if r.rows_affected() == 0 => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "Resource not found" }))).into_response()
        }
        Ok(r) => (StatusCode::OK, Json(json!({ "updated": r.rows_affected() }))).into_response(),
        Err(sqlx::Error::Database(e)) => {
            (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// PATCH /api/resources/{id}
pub async fn edit(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<i32>,
    Json(edit): Json<Edit>,
) -> impl IntoResponse {
    if let Err(e) = adminonly(&state, &claims).await {
        return e;
    }
    save(&state, vec![id], edit).await
}

/// PATCH /api/resources — same edit on every id in `ids`
pub async fn editmany(State(state): State<AppState>, claims: Claims, Json(mut edit): Json<Edit>) -> impl IntoResponse {
    if let Err(e) = adminonly(&state, &claims).await {
        return e;
    }
    let ids = std::mem::take(&mut edit.ids);
    save(&state, ids, edit).await
}

/// reruns the rules on the sources holding `ids`, or on every active source, returns rows changed
async fn rerun(state: &AppState, ids: Option<&[i32]>) -> Result<u64, sqlx::Error> {
    let sources: Vec<(i32, String, serde_json::Value)> = sqlx::query_as(
        "SELECT id, repo, aliases FROM sources WHERE source_status NOT IN ('archived', 'error') \
         AND ($1::int[] IS NULL OR id IN (SELECT source_id FROM resources WHERE id = ANY($1)))",
    )
    .bind(ids)
    .fetch_all(&state.db)
    .await?;
    let mut changed = 0;
    for (id, repo, aliases) in sources {
        let aliases: HashMap<String, String> = serde_json::from_value(aliases).unwrap_or_default();
        let mut tx = state.db.begin().await?;
        changed += classify::apply(&mut tx, &state.courses, id, &repo, &aliases).await?;
        tx.commit().await?;
    }
    Ok(changed)
}

/// POST /api/reclassify — rerun rules on every active source without syncing
pub async fn reclassify(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    if let Err(e) = adminonly(&state, &claims).await {
        return e;
    }
    match rerun(&state, None).await {
        Ok(n) => (StatusCode::OK, Json(json!({ "updated": n }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
