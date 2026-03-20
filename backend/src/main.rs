mod auth;
mod config;
mod db;
mod github;
mod routes;
mod search;

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
};
use meilisearch_sdk::client::Client;
use sqlx::PgPool;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub meili: Client,
    pub config: config::Config,
}

async fn require_json_for_mutations(req: Request, next: Next) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    if method == axum::http::Method::POST || method == axum::http::Method::DELETE || method == axum::http::Method::PUT || method == axum::http::Method::PATCH {
        let content_type = req.headers().get(axum::http::header::CONTENT_TYPE);
        if let Some(ct) = content_type {
            if !ct.to_str().unwrap_or("").starts_with("application/json") {
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        } else {
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    }
    Ok(next.run(req).await)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env();
    let db = db::init_pool(&config).await;
    let meili = search::init_client(&config);

    search::init_indexes(&meili, &db).await;

    let origins: Vec<axum::http::HeaderValue> = config
        .allowed_origins
        .iter()
        .map(|o| o.parse().unwrap())
        .collect();

    let state = AppState {
        db,
        meili,
        config: config.clone(),
    };

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/api/auth/login", get(routes::auth::login))
        .route("/api/auth/callback", get(routes::auth::callback))
        .route("/api/auth/me", get(routes::auth::me))
        .route("/api/auth/logout", post(routes::auth::logout))
        .route("/api/sources", post(routes::sources::create_source))
        .route("/api/sources", get(routes::sources::list_sources))
        .route("/api/sources/{id}/sync", post(routes::sources::sync_source))
        .route("/api/sources/{id}", delete(routes::sources::delete_source))
        .route("/api/search", get(routes::search::search))
        .route(
            "/api/sources/{id}/like",
            post(routes::likes::toggle_source_like),
        )
        .route("/api/resources", get(routes::resources::list_resources))
        .route("/api/resources/{id}/like", post(routes::likes::toggle_like))
        .route(
            "/api/resources/{id}",
            delete(routes::resources::delete_resource),
        )
        .route("/api/resources/{id}/likes", get(routes::likes::get_likes))
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(middleware::from_fn(require_json_for_mutations))
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_credentials(true)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
