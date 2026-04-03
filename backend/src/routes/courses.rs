use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::routes::auth::{OptionalAuth, upsertuser};
use ocas_auth::Claims;

#[derive(Debug, Serialize)]
pub struct CourseRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub resource_count: i64,
    pub pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub q: Option<String>,
}

pub async fn listcourses(
    State(state): State<AppState>,
    auth: OptionalAuth,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(30).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let search = params.q.unwrap_or_default().to_lowercase();
    let search_pattern = format!("%{}%", search);

    let current_user_id = match &auth {
        OptionalAuth::Authenticated(u) => Some(u.id),
        OptionalAuth::Anonymous => None,
    };

    let pinned = match current_user_id {
        Some(uid) => {
            sqlx::query_as!(
                CourseRow,
                r#"
                SELECT c.id, c.name, c.aliases, COUNT(r.id) as "resource_count!",
                    true as "pinned!"
                FROM courses c
                LEFT JOIN resources r ON r.course_id = c.id
                INNER JOIN coursepins cp ON cp.course_id = c.id AND cp.user_id = $1
                WHERE ($2 = '' OR c.name ILIKE $3 OR EXISTS (
                    SELECT 1 FROM unnest(c.aliases) a WHERE a ILIKE $3
                ))
                GROUP BY c.id
                ORDER BY c.name
                "#,
                uid,
                search,
                search_pattern
            )
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
        }
        None => vec![],
    };

    let pinned_ids: Vec<uuid::Uuid> = pinned.iter().map(|c| c.id).collect();

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM courses c
        WHERE ($1 = '' OR c.name ILIKE $2 OR EXISTS (
            SELECT 1 FROM unnest(c.aliases) a WHERE a ILIKE $2
        ))
        AND c.id != ALL($3)
        "#,
        search,
        search_pattern,
        &pinned_ids
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let courses = match current_user_id {
        Some(uid) => {
            sqlx::query_as!(
                CourseRow,
                r#"
                SELECT c.id, c.name, c.aliases, COUNT(r.id) as "resource_count!",
                    EXISTS(SELECT 1 FROM coursepins cp WHERE cp.course_id = c.id AND cp.user_id = $5) as "pinned!"
                FROM courses c
                LEFT JOIN resources r ON r.course_id = c.id
                WHERE ($1 = '' OR c.name ILIKE $2 OR EXISTS (
                    SELECT 1 FROM unnest(c.aliases) a WHERE a ILIKE $2
                ))
                AND c.id != ALL($6)
                GROUP BY c.id
                ORDER BY c.name
                LIMIT $3 OFFSET $4
                "#,
                search,
                search_pattern,
                per_page,
                offset,
                uid,
                &pinned_ids
            )
            .fetch_all(&state.db)
            .await
        }
        None => {
            sqlx::query_as!(
                CourseRow,
                r#"
                SELECT c.id, c.name, c.aliases, COUNT(r.id) as "resource_count!",
                    false as "pinned!"
                FROM courses c
                LEFT JOIN resources r ON r.course_id = c.id
                WHERE ($1 = '' OR c.name ILIKE $2 OR EXISTS (
                    SELECT 1 FROM unnest(c.aliases) a WHERE a ILIKE $2
                ))
                AND c.id != ALL($5)
                GROUP BY c.id
                ORDER BY c.name
                LIMIT $3 OFFSET $4
                "#,
                search,
                search_pattern,
                per_page,
                offset,
                &pinned_ids
            )
            .fetch_all(&state.db)
            .await
        }
    };

    match courses {
        Ok(courses) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "courses": courses,
                "pinned": pinned,
                "total": total,
                "page": page,
                "per_page": per_page,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn togglepin(
    State(state): State<AppState>,
    claims: Claims,
    Path(course_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    let existing = sqlx::query_scalar!(
        r#"SELECT 1 as "exists!" FROM coursepins WHERE user_id = $1 AND course_id = $2"#,
        user.id,
        course_id
    )
    .fetch_optional(&state.db)
    .await;

    let pinned = match existing {
        Ok(Some(_)) => {
            let _ = sqlx::query!(
                "DELETE FROM coursepins WHERE user_id = $1 AND course_id = $2",
                user.id,
                course_id
            )
            .execute(&state.db)
            .await;
            false
        }
        _ => {
            let result = sqlx::query!(
                "INSERT INTO coursepins (user_id, course_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                user.id,
                course_id
            )
            .execute(&state.db)
            .await;

            match result {
                Ok(_) => true,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                        .into_response()
                }
            }
        }
    };

    (StatusCode::OK, Json(serde_json::json!({ "pinned": pinned }))).into_response()
}

#[derive(Debug, Serialize)]
struct CourseInfo {
    id: uuid::Uuid,
    name: String,
    aliases: Vec<String>,
}

pub async fn getcourse(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let course = sqlx::query_as!(
        CourseInfo,
        "SELECT id, name, aliases FROM courses WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await;

    let course = match course {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "course not found" })),
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

    let resources = sqlx::query_as!(
        super::resources::ResourceRow,
        r#"
        SELECT r.id, r.source_id, s.owner, s.repo, s.branch, r.file_path, r.title, r.type, r.like_count
        FROM resources r
        JOIN sources s ON r.source_id = s.id
        WHERE r.course_id = $1 AND s.source_status NOT IN ('archived', 'error')
        "#,
        id
    )
    .fetch_all(&state.db)
    .await;

    match resources {
        Ok(resources) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "course": course,
                "resources": resources,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
