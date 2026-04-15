mod config;
mod courses;
mod db;
mod github;
mod routes;
mod search;

#[cfg(all(feature = "ocas", feature = "cas"))]
compile_error!("features `ocas` and `cas` are mutually exclusive");

#[cfg(not(any(feature = "ocas", feature = "cas")))]
compile_error!("one of `ocas` or `cas` must be enabled");

use axum::{
    Router,
    routing::{delete, get, post},
};
use meilisearch_sdk::client::Client;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub meili: Client,
    pub config: config::Config,
    pub courses: courses::CourseRegistry,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = config::Config::from_env()?;
    let db = db::init_pool(&config).await?;
    let meili = search::init_client(&config)?;

    search::init_indexes(&meili, &db).await;

    #[cfg(all(feature = "ocas", not(feature = "mock")))]
    let ocasclient = ocas_auth::OcasClient::new(ocas_auth::OcasConfig::fromenv()).await;

    #[cfg(all(feature = "ocas", feature = "mock"))]
    let ocasclient = {
        tracing::info!("Mock feature enabled, overriding OCAS_URL to point to localhost");
        let mut c = ocas_auth::OcasConfig::fromenv();
        c.url = format!("http://localhost:{}", config.server_port);
        ocas_auth::OcasClient::new(c).await
    };

    let origins: Vec<axum::http::HeaderValue> = config
        .allowed_origins
        .iter()
        .map(|o| o.parse())
        .collect::<Result<_, _>>()?;

    let courses = courses::CourseRegistry::load(&db).await?;

    let state = AppState {
        db,
        meili,
        config: config.clone(),
        courses,
    };

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/api/auth/me", get(routes::auth::me))
        .route("/api/auth/mock/login", get(routes::auth::mock_login))
        .route("/api/sources", post(routes::sources::create_source))
        .route("/api/sources", get(routes::sources::list_sources))
        .route("/api/sources/{id}/sync", post(routes::sources::sync_source))
        .route("/api/sources/{id}", delete(routes::sources::delete_source))
        .route("/api/search", get(routes::search::search))
        .route("/api/courses", get(routes::courses::listcourses))
        .route("/api/courses/{slug}", get(routes::courses::getcourse))
        .route("/api/courses/{slug}/pin", post(routes::courses::togglepin))
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
        .route("/api/resources/{id}/likes", get(routes::likes::get_likes));

    #[cfg(feature = "cas")]
    let app = app
        .route("/api/auth/login", get(routes::cas::login))
        .route("/api/auth/callback", get(routes::cas::callback))
        .route("/api/auth/logout", get(routes::cas::logout));

    let app = app.with_state(state);

    #[cfg(feature = "ocas")]
    let app = app.nest("/api/auth", ocasclient.authrouter());

    #[cfg(all(feature = "ocas", feature = "mock"))]
    let app = app.merge(ocas_auth::mock::mockjwks());

    #[cfg(feature = "ocas")]
    let app = app
        .layer(axum::middleware::from_fn(ocas_auth::refresh::refreshmw))
        .layer(ocasclient.layer());

    let app = app
        .layer(tower_http::compression::CompressionLayer::new())
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
        );

    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
