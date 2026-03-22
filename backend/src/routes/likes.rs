use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::AppState;
use crate::routes::auth::upsertuser;
use ocas_auth::Claims;

async fn toggle_generic_like(
    state: &AppState,
    user_id: uuid::Uuid,
    entity_id: i32,
    like_table: &str,
    entity_table: &str,
    fk_column: &str,
) -> Result<(bool, i32), (StatusCode, Json<serde_json::Value>)> {
    let mut tx = state.db.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let check_query = format!("SELECT 1 as exists FROM {} WHERE user_id = $1 AND {} = $2", like_table, fk_column);
    let existing: Option<i32> = sqlx::query_scalar(&check_query)
        .bind(user_id)
        .bind(entity_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    let liked;

    if existing.is_some() {
        let delete_query = format!("DELETE FROM {} WHERE user_id = $1 AND {} = $2", like_table, fk_column);
        sqlx::query(&delete_query)
            .bind(user_id)
            .bind(entity_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
            })?;
        liked = false;
    } else {
        let insert_query = format!("INSERT INTO {} (user_id, {}) VALUES ($1, $2)", like_table, fk_column);
        let insert = sqlx::query(&insert_query)
            .bind(user_id)
            .bind(entity_id)
            .execute(&mut *tx)
            .await;

        if insert.is_err() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("{} not found", entity_table) })),
            ));
        }
        liked = true;
    }

    let update_query = format!(
        "UPDATE {} SET like_count = (SELECT COUNT(*)::int FROM {} WHERE {} = $1) WHERE id = $1",
        entity_table, like_table, fk_column
    );

    let _ = sqlx::query(&update_query)
        .bind(entity_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        })?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
    })?;

    let get_count_query = format!("SELECT like_count FROM {} WHERE id = $1", entity_table);
    let like_count: i32 = sqlx::query_scalar(&get_count_query)
        .bind(entity_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok((liked, like_count))
}

pub async fn toggle_like(
    State(state): State<AppState>,
    claims: Claims,
    Path(resource_id): Path<i32>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    match toggle_generic_like(&state, user.id, resource_id, "likes", "resources", "resource_id").await {
        Ok((liked, like_count)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "liked": liked, "like_count": like_count })),
        ).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_likes(
    State(state): State<AppState>,
    Path(resource_id): Path<i32>,
) -> impl IntoResponse {
    let like_count = sqlx::query_scalar!(
        r#"SELECT like_count FROM resources WHERE id = $1"#,
        resource_id
    )
    .fetch_optional(&state.db)
    .await;

    match like_count {
        Ok(Some(count)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "like_count": count })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Resource not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn toggle_source_like(
    State(state): State<AppState>,
    claims: Claims,
    Path(source_id): Path<i32>,
) -> impl IntoResponse {
    let user = match upsertuser(&state, &claims).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };

    match toggle_generic_like(&state, user.id, source_id, "source_likes", "sources", "source_id").await {
        Ok((liked, like_count)) => (
            StatusCode::OK,
            Json(serde_json::json!({ "liked": liked, "like_count": like_count })),
        ).into_response(),
        Err(e) => e.into_response(),
    }
}
