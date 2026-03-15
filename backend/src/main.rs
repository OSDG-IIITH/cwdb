mod auth;
mod config;
mod db;
mod github;
mod routes;
mod search;

use axum::{
    Router,
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
        .route("/api/test/index", post(routes::test::index_document))
        .route("/api/test/search", get(routes::test::search_documents))
        .route("/api/test/resource", post(routes::test::create_resource))
        .route("/api/test/resources", get(routes::test::list_resources))
        .route("/api/test/sync", post(routes::test::sync_resource))
        .route("/api/test/github-tree", get(routes::test::github_tree))
        .route("/api/sources", post(routes::sources::create_source))
        .route("/api/sources", get(routes::sources::list_sources))
        .route("/api/sources/{id}/sync", post(routes::sources::sync_source))
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
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173"
                        .parse::<axum::http::HeaderValue>()
                        .unwrap(),
                    "http://127.0.0.1:5173"
                        .parse::<axum::http::HeaderValue>()
                        .unwrap(),
                ])
                .allow_credentials(true)
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
