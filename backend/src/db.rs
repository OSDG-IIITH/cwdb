use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Config;

pub async fn init_pool(config: &Config) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    Ok(pool)
}
