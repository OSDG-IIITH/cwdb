use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::config::Config;

const MS_AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const MS_JWKS_URL: &str = "https://login.microsoftonline.com/common/discovery/v2.0/keys";

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
    pub tid: String,
    pub iss: String,
    pub nonce: Option<String>,
    pub exp: i64,
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid domain")]
    InvalidDomain,
    #[error("Missing email claim")]
    MissingEmail,
    #[error("Missing nonce claim")]
    MissingNonce,
    #[error("Invalid nonce")]
    InvalidNonce,
    #[error("Token exchange failed: {0}")]
    TokenExchange(String),
    #[error("Failed to fetch signing keys: {0}")]
    JwksFetch(String),
    #[error("Missing key id in token header")]
    MissingKid,
    #[error("No matching signing key found")]
    SigningKeyNotFound,
    #[error("Invalid issuer")]
    InvalidIssuer,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

pub fn build_login_url(config: &Config, state: &str, nonce: &str) -> String {
    let params = [
        ("client_id", config.ms_client_id.as_str()),
        ("response_type", "code"),
        ("redirect_uri", config.ms_redirect_uri.as_str()),
        ("response_mode", "query"),
        ("scope", "openid profile email"),
        ("state", state),
        ("nonce", nonce),
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

pub async fn decode_id_token(
    config: &Config,
    id_token: &str,
    expected_nonce: Option<&str>,
) -> Result<IdTokenClaims, AuthError> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::InvalidToken("Invalid JWT format".into()));
    }

    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    let _: IdTokenClaims =
        serde_json::from_slice(&payload).map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    let header = decode_header(id_token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;
    let kid = header.kid.ok_or(AuthError::MissingKid)?;

    let client = reqwest::Client::new();
    let jwks: JwksResponse = client
        .get(MS_JWKS_URL)
        .send()
        .await
        .map_err(|e| AuthError::JwksFetch(e.to_string()))?
        .error_for_status()
        .map_err(|e| AuthError::JwksFetch(e.to_string()))?
        .json()
        .await
        .map_err(|e| AuthError::JwksFetch(e.to_string()))?;

    let jwk = jwks
        .keys
        .into_iter()
        .find(|k| k.kid == kid && k.kty == "RSA")
        .ok_or(AuthError::SigningKeyNotFound)?;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.ms_client_id.as_str()]);
    validation.required_spec_claims = HashSet::from([
        "exp".to_string(),
        "aud".to_string(),
        "iss".to_string(),
        "sub".to_string(),
    ]);

    let claims = decode::<IdTokenClaims>(id_token, &decoding_key, &validation)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?
        .claims;

    let expected_v2_issuer = format!("https://login.microsoftonline.com/{}/v2.0", claims.tid);
    let expected_v1_issuer = format!("https://sts.windows.net/{}/", claims.tid);
    if claims.iss != expected_v2_issuer && claims.iss != expected_v1_issuer {
        return Err(AuthError::InvalidIssuer);
    }

    if let Some(expected_nonce) = expected_nonce {
        let nonce = claims.nonce.as_deref().ok_or(AuthError::MissingNonce)?;
        if nonce != expected_nonce {
            return Err(AuthError::InvalidNonce);
        }
    }

    Ok(claims)
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
