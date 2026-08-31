use std::{collections::HashMap, time::Duration};

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

pub async fn verify_id_token(
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
