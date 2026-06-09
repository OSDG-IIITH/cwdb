use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub admin_emails: Vec<String>,
    pub frontend_url: String,
    pub allowed_origins: Vec<String>,
    pub github_token: Option<String>,
    pub publish_path: String,
    #[cfg(feature = "cas")]
    pub cas_base_url: String,
    #[cfg(feature = "cas")]
    pub cas_service_url: String,
    #[cfg(feature = "cas")]
    pub session_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?,
            server_port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .map_err(|_| "PORT must be a number")?,
            admin_emails: env::var("ADMIN_EMAILS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
            github_token: env::var("GITHUB_TOKEN").ok(),
            publish_path: env::var("PUBLISH_PATH").unwrap_or_else(|_| "/app/data".into()),
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|_| vec!["http://localhost:5173".to_string(), "http://127.0.0.1:5173".to_string()]),
            #[cfg(feature = "cas")]
            cas_base_url: env::var("CAS_BASE_URL")
                .unwrap_or_else(|_| "https://login.iiit.ac.in/cas".into()),
            #[cfg(feature = "cas")]
            cas_service_url: env::var("CAS_SERVICE_URL")
                .map_err(|_| "CAS_SERVICE_URL must be set")?,
            #[cfg(feature = "cas")]
            session_secret: env::var("SESSION_SECRET")
                .map_err(|_| "SESSION_SECRET must be set")?,
        })
    }
}
