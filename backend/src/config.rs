use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub meili_host: String,
    pub meili_key: Option<String>,
    pub server_port: u16,
    pub ms_client_id: String,
    pub ms_client_secret: String,
    pub ms_redirect_uri: String,
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
            ms_client_id: env::var("MS_CLIENT_ID").expect("MS_CLIENT_ID must be set"),
            ms_client_secret: env::var("MS_CLIENT_SECRET").expect("MS_CLIENT_SECRET must be set"),
            ms_redirect_uri: env::var("MS_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/api/auth/callback".into()),
        }
    }
}
