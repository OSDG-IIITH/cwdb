use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Config;

pub async fn init_pool(config: &Config) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres")
}
