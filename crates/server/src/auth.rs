use std::{collections::HashMap, time::Duration};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub uid: String,
}

#[derive(Debug, Deserialize)]
struct FirebaseClaims {
    sub: String,
    aud: String,
    iss: Option<String>,
    exp: Option<u64>,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("missing key id")]
    MissingKid,
    #[error("unknown key id")]
    UnknownKid,
    #[error("invalid token")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
    #[error("http error")]
    Http(#[from] reqwest::Error),
    #[error("invalid claims")]
    InvalidClaims,
}

pub fn auth_emulator_enabled() -> bool {
    std::env::var("FIREBASE_AUTH_EMULATOR_HOST")
        .ok()
        .is_some_and(|v| !v.trim().is_empty())
}

pub async fn verify_id_token(
    project_id: &str,
    token: &str,
) -> Result<AuthenticatedUser, AuthError> {
    if auth_emulator_enabled() {
        return verify_emulator_id_token(project_id, token);
    }
    verify_production_id_token(project_id, token).await
}

fn verify_emulator_id_token(project_id: &str, token: &str) -> Result<AuthenticatedUser, AuthError> {
    let payload_b64 = token.split('.').nth(1).ok_or(AuthError::InvalidClaims)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| AuthError::InvalidClaims)?;
    let claims: FirebaseClaims =
        serde_json::from_slice(&bytes).map_err(|_| AuthError::InvalidClaims)?;

    if claims.aud != project_id {
        return Err(AuthError::InvalidClaims);
    }
    let expected_iss = format!("https://securetoken.google.com/{project_id}");
    if claims.iss.as_deref() != Some(expected_iss.as_str()) {
        return Err(AuthError::InvalidClaims);
    }
    if let Some(exp) = claims.exp {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| AuthError::InvalidClaims)?
            .as_secs();
        if exp + 60 < now {
            return Err(AuthError::InvalidClaims);
        }
    }

    Ok(AuthenticatedUser { uid: claims.sub })
}

async fn verify_production_id_token(
    project_id: &str,
    token: &str,
) -> Result<AuthenticatedUser, AuthError> {
    let header = decode_header(token).map_err(AuthError::InvalidToken)?;
    let kid = header.kid.ok_or(AuthError::MissingKid)?;
    let cert = fetch_google_cert(&kid).await?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[project_id]);
    validation.set_issuer(&[format!("https://securetoken.google.com/{project_id}")]);

    let token_data = decode::<FirebaseClaims>(token, &cert, &validation)?;
    if token_data.claims.aud != project_id {
        return Err(AuthError::InvalidClaims);
    }

    Ok(AuthenticatedUser {
        uid: token_data.claims.sub,
    })
}

async fn fetch_google_cert(kid: &str) -> Result<DecodingKey, AuthError> {
    let response: HashMap<String, String> = Client::new()
        .get("https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com")
        .timeout(Duration::from_secs(5))
        .send()
        .await?
        .json()
        .await?;

    let pem = response.get(kid).ok_or(AuthError::UnknownKid)?;
    DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(AuthError::InvalidToken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emulator_enabled_when_host_set() {
        std::env::set_var("FIREBASE_AUTH_EMULATOR_HOST", "127.0.0.1:9099");
        assert!(auth_emulator_enabled());
        std::env::remove_var("FIREBASE_AUTH_EMULATOR_HOST");
    }
}
