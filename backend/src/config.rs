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
    pub admin_emails: Vec<String>,
    pub frontend_url: String,
    pub allowed_origins: Vec<String>,
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?,
            meili_host: env::var("MEILI_HOST").unwrap_or_else(|_| "http://localhost:7700".into()),
            meili_key: env::var("MEILI_MASTER_KEY").ok(),
            server_port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .map_err(|_| "PORT must be a number")?,
            ms_client_id: env::var("MS_CLIENT_ID").map_err(|_| "MS_CLIENT_ID must be set")?,
            ms_client_secret: env::var("MS_CLIENT_SECRET").map_err(|_| "MS_CLIENT_SECRET must be set")?,
            ms_redirect_uri: env::var("MS_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/api/auth/callback".into()),
            admin_emails: env::var("ADMIN_EMAILS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|_| vec!["http://localhost:5173".to_string(), "http://127.0.0.1:5173".to_string()]),
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|s| s.to_lowercase() == "true")
                .unwrap_or(false),
        })
    }
}
