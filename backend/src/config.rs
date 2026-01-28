use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub meili_host: String,
    pub meili_key: Option<String>,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            meili_host: env::var("MEILI_HOST").unwrap_or_else(|_| "http://localhost:7700".into()),
            meili_key: env::var("MEILI_MASTER_KEY").ok(),
            server_port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a number"),
        }
    }
}
