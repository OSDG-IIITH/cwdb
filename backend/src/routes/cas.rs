#[cfg(feature = "cas")]
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
#[cfg(feature = "cas")]
use chrono::Utc;
#[cfg(feature = "cas")]
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
#[cfg(feature = "cas")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "cas")]
use uuid::Uuid;

#[cfg(feature = "cas")]
use crate::AppState;
#[cfg(feature = "cas")]
use crate::config::Config;
#[cfg(feature = "cas")]
use super::auth::{AuthError, Claims};

#[cfg(feature = "cas")]
#[derive(Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    email: String,
    exp: usize,
}

#[cfg(feature = "cas")]
#[derive(Deserialize)]
struct CasServiceResponse {
    #[serde(rename = "serviceResponse")]
    service_response: ServiceResponse,
}

#[cfg(feature = "cas")]
#[derive(Deserialize)]
struct ServiceResponse {
    #[serde(rename = "authenticationSuccess")]
    authentication_success: Option<AuthSuccess>,
    #[serde(rename = "authenticationFailure")]
    authentication_failure: Option<AuthFailure>,
}

#[cfg(feature = "cas")]
#[derive(Deserialize)]
struct AuthSuccess {
    attributes: CasAttributes,
}

#[cfg(feature = "cas")]
#[derive(Deserialize)]
struct CasAttributes {
    uid: Vec<String>,
    #[serde(rename = "E-Mail")]
    email: Vec<String>,
}

#[cfg(feature = "cas")]
#[derive(Deserialize)]
struct AuthFailure {
    description: String,
}

#[cfg(feature = "cas")]
#[derive(Deserialize)]
pub struct CallbackParams {
    ticket: Option<String>,
}

/// issue a cwdb_session JWT (7-day lifetime)
#[cfg(feature = "cas")]
pub fn issue_session(sub: Uuid, email: &str, secret: &str) -> String {
    let exp = (Utc::now() + chrono::Duration::days(7)).timestamp() as usize;
    let claims = JwtClaims { sub: sub.to_string(), email: email.to_string(), exp };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .unwrap_or_default()
}

/// validate a cwdb_session JWT and return Claims
#[cfg(feature = "cas")]
pub fn validate_session(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

    let sub = Uuid::parse_str(&data.claims.sub)
        .map_err(|e| AuthError::Unauthorized(e.to_string()))?;

    Ok(Claims { sub, email: data.claims.email })
}

#[cfg(feature = "cas")]
async fn validate_ticket(ticket: &str, config: &Config) -> Result<(Uuid, String), AuthError> {
    let resp = reqwest::Client::new()
        .get(format!("{}/serviceValidate", config.cas_base_url))
        .query(&[
            ("service", config.cas_service_url.as_str()),
            ("ticket", ticket),
            ("format", "JSON"),
        ])
        .send()
        .await
        .map_err(|e| AuthError::Unauthorized(format!("CAS request failed: {}", e)))?;

    let body: CasServiceResponse = resp
        .json()
        .await
        .map_err(|e| AuthError::Unauthorized(format!("CAS response parse failed: {}", e)))?;

    match body.service_response.authentication_success {
        Some(success) => {
            let uid = success
                .attributes
                .uid
                .into_iter()
                .next()
                .ok_or_else(|| AuthError::Unauthorized("no uid in CAS response".into()))?;
            let email = success
                .attributes
                .email
                .into_iter()
                .next()
                .ok_or_else(|| AuthError::Unauthorized("no email in CAS response".into()))?;
            let sub = Uuid::new_v5(&Uuid::NAMESPACE_URL, uid.as_bytes());
            Ok((sub, email))
        }
        None => {
            let reason = body
                .service_response
                .authentication_failure
                .map(|f| f.description)
                .unwrap_or_else(|| "authentication failed".into());
            Err(AuthError::Unauthorized(reason))
        }
    }
}

/// GET /api/auth/login — redirect to CAS
#[cfg(feature = "cas")]
pub async fn login(State(state): State<AppState>) -> Redirect {
    let url = format!(
        "{}/login?service={}",
        state.config.cas_base_url,
        urlencoding(&state.config.cas_service_url)
    );
    Redirect::to(&url)
}

/// GET /api/auth/callback?ticket=ST-xxx — validate ticket, set session cookie
#[cfg(feature = "cas")]
pub async fn callback(
    Query(params): Query<CallbackParams>,
    State(state): State<AppState>,
) -> Response {
    let ticket = match params.ticket {
        Some(t) => t,
        None => {
            return Redirect::to(&format!("{}?error=missing_ticket", state.config.frontend_url))
                .into_response()
        }
    };

    let (sub, email) = match validate_ticket(&ticket, &state.config).await {
        Ok(v) => v,
        Err(e) => {
            let AuthError::Unauthorized(msg) = e;
            return Redirect::to(&format!(
                "{}?error={}",
                state.config.frontend_url,
                urlencoding(&msg)
            ))
            .into_response();
        }
    };

    let claims = Claims { sub, email };
    if let Err(e) = super::auth::upsertuser(&state, &claims).await {
        let AuthError::Unauthorized(msg) = e;
        return Redirect::to(&format!(
            "{}?error={}",
            state.config.frontend_url,
            urlencoding(&msg)
        ))
        .into_response();
    }

    let token = issue_session(sub, &claims.email, &state.config.session_secret);
    let mut resp = Redirect::to(&state.config.frontend_url).into_response();
    let cookie = format!(
        "cwdb_session={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
        token,
        7 * 24 * 3600
    );
    resp.headers_mut()
        .append(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    resp
}

/// GET /api/auth/logout — clear session cookie and redirect
#[cfg(feature = "cas")]
pub async fn logout(State(state): State<AppState>) -> Response {
    let mut resp = Redirect::to(&state.config.frontend_url).into_response();
    let cookie = "cwdb_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0";
    resp.headers_mut()
        .append(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    resp
}

/// percent-encode a string for use in URL query values
#[cfg(feature = "cas")]
fn urlencoding(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            _ => format!("%{:02X}", c as u32).chars().collect::<Vec<_>>(),
        })
        .collect()
}
