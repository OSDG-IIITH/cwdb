use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;

use crate::AppState;
use crate::classify;
use crate::routes::auth::{Claims, adminonly};

/// GET /api/review — rows flagged for review, and unclassified row counts per source
pub async fn review(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    if let Err(e) = adminonly(&state, &claims).await {
        return e;
    }

    let run = async {
        let rows = sqlx::query(
            "SELECT r.id, r.source_id, s.repo, r.file_path, r.course, r.kind, r.examtype, r.slidetype, r.n, r.key, \
             r.confidence, o.season::text AS season, o.year::int AS year \
             FROM resources r JOIN sources s ON s.id = r.source_id LEFT JOIN offerings o ON o.id = r.offering \
             WHERE r.needsreview AND NOT r.hidden ORDER BY r.confidence NULLS FIRST, r.file_path",
        )
        .fetch_all(&state.db)
        .await?;
        let new: Vec<(i32, i64)> = sqlx::query_as(
            "SELECT source_id, count(*) FROM resources WHERE NOT hidden \
             AND classifiedby IS DISTINCT FROM 'manual' AND classifiedby IS DISTINCT FROM 'llm' GROUP BY source_id",
        )
        .fetch_all(&state.db)
        .await?;

        let rows: Vec<_> = rows
            .iter()
            .map(|r| {
                let path: String = r.get("file_path");
                let course: Option<String> = r.get("course");
                let term = course.as_deref().and_then(|c| state.courses.term(c));
                let g = classify::classify(&path, r.get("repo"), term);
                let rule = match (g.kind, g.examtype) {
                    (Some(k), Some(t)) => format!("{k} · {t}"),
                    (Some(k), None) => k.to_string(),
                    _ => "—".into(),
                };
                let conf: Option<f32> = r.get("confidence");
                let mut why = Vec::new();
                if g.kind.is_some() && g.kind != r.get::<Option<String>, _>("kind").as_deref() {
                    why.push(format!("rule said {rule}"));
                }
                if conf.is_some_and(|c| c < 0.8) {
                    why.push("low confidence".into());
                }
                json!({
                    "id": r.get::<i32, _>("id"), "source": r.get::<i32, _>("source_id"), "path": path,
                    "course": course, "kind": r.get::<Option<String>, _>("kind"),
                    "examtype": r.get::<Option<String>, _>("examtype"), "slidetype": r.get::<Option<String>, _>("slidetype"),
                    "n": r.get::<Option<i32>, _>("n"), "key": r.get::<bool, _>("key"),
                    "season": r.get::<Option<String>, _>("season"), "year": r.get::<Option<i32>, _>("year"),
                    "conf": conf, "rule": rule, "why": why.join(" · "),
                })
            })
            .collect();
        Ok::<_, sqlx::Error>(json!({ "rows": rows, "new": new.into_iter().collect::<HashMap<_, _>>() }))
    };

    match run.await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
