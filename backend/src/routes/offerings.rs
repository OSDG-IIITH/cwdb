use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;
use crate::classify;
use crate::routes::auth::{Claims, adminonly};

#[derive(Deserialize)]
pub struct NewOffering {
    pub course: String,
    pub season: String,
    pub year: i32,
}

/// POST /api/offerings — create a historical offering, or return the existing one
pub async fn create(State(state): State<AppState>, claims: Claims, Json(o): Json<NewOffering>) -> impl IntoResponse {
    if let Err(e) = adminonly(&state, &claims).await {
        return e;
    }

    let mut db = match state.db.acquire().await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    };

    match classify::offering(&mut db, &o.course, &o.season, o.year).await {
        Ok(id) => (StatusCode::CREATED, Json(json!({ "id": id }))).into_response(),
        Err(sqlx::Error::Database(e)) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}
