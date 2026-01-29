use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};

use crate::config::Config;

const MS_AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";

const ALLOWED_DOMAINS: &[&str] = &[
    "@iiit.ac.in",
    "@students.iiit.ac.in",
    "@research.iiit.ac.in",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub name: Option<String>,
    pub sub: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid domain")]
    InvalidDomain,
    #[error("Missing email claim")]
    MissingEmail,
    #[error("Token exchange failed: {0}")]
    TokenExchange(String),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

pub fn build_login_url(config: &Config) -> String {
    let params = [
        ("client_id", config.ms_client_id.as_str()),
        ("response_type", "code"),
        ("redirect_uri", config.ms_redirect_uri.as_str()),
        ("response_mode", "query"),
        ("scope", "openid profile email"),
    ];

    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", MS_AUTH_URL, query)
}

pub async fn exchange_code(config: &Config, code: &str) -> Result<TokenResponse, AuthError> {
    let client = reqwest::Client::new();

    let params = [
        ("client_id", config.ms_client_id.as_str()),
        ("client_secret", config.ms_client_secret.as_str()),
        ("code", code),
        ("redirect_uri", config.ms_redirect_uri.as_str()),
        ("grant_type", "authorization_code"),
    ];

    let response = client
        .post(MS_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| AuthError::TokenExchange(e.to_string()))?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(AuthError::TokenExchange(text));
    }

    response
        .json()
        .await
        .map_err(|e| AuthError::TokenExchange(e.to_string()))
}

pub fn decode_id_token(id_token: &str) -> Result<IdTokenClaims, AuthError> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::InvalidToken("Invalid JWT format".into()));
    }

    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    serde_json::from_slice(&payload).map_err(|e| AuthError::InvalidToken(e.to_string()))
}

pub fn extract_email(claims: &IdTokenClaims) -> Result<String, AuthError> {
    claims
        .email
        .clone()
        .or_else(|| claims.preferred_username.clone())
        .ok_or(AuthError::MissingEmail)
}

pub fn validate_domain(email: &str) -> Result<(), AuthError> {
    let email_lower = email.to_lowercase();
    if ALLOWED_DOMAINS.iter().any(|d| email_lower.ends_with(d)) {
        Ok(())
    } else {
        Err(AuthError::InvalidDomain)
    }
}
