use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::error::{ApiError, ApiResult};

const FIREBASE_CERTS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

#[derive(Debug, Clone)]
pub struct VerifiedUser {
    pub uid: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FirebaseClaims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
}

pub struct FirebaseVerifier {
    project_id: String,
    keys: Arc<RwLock<HashMap<String, String>>>,
}

impl FirebaseVerifier {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn refresh_keys(&self) -> ApiResult<()> {
        let resp = attohttpc::get(FIREBASE_CERTS_URL)
            .send()
            .map_err(|_| ApiError::Internal)?;
        if resp.status() != 200 {
            return Err(ApiError::Internal);
        }
        let map: HashMap<String, String> = resp.json().map_err(|_| ApiError::Internal)?;
        *self.keys.write().map_err(|_| ApiError::Internal)? = map;
        Ok(())
    }

    pub async fn verify(self: &Arc<Self>, token: &str) -> ApiResult<VerifiedUser> {
        let this = Arc::clone(self);
        let token = token.to_string();
        tokio::task::spawn_blocking(move || this.verify_blocking(&token))
            .await
            .map_err(|_| ApiError::Internal)?
    }

    fn verify_blocking(&self, token: &str) -> ApiResult<VerifiedUser> {
        let header = decode_header(token).map_err(|_| ApiError::Unauthorized)?;
        let kid = header.kid.ok_or(ApiError::Unauthorized)?;

        let pem = {
            let keys = self.keys.read().map_err(|_| ApiError::Internal)?;
            keys.get(&kid).cloned()
        };

        let pem = match pem {
            Some(p) => p,
            None => {
                self.refresh_keys()?;
                self.keys
                    .read()
                    .map_err(|_| ApiError::Internal)?
                    .get(&kid)
                    .cloned()
                    .ok_or(ApiError::Unauthorized)?
            }
        };

        let issuer = format!("https://securetoken.google.com/{}", self.project_id);
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.project_id]);
        validation.iss = Some(issuer);

        let decoded = decode::<FirebaseClaims>(
            token,
            &DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(|_| ApiError::Internal)?,
            &validation,
        )
        .map_err(|_| ApiError::Unauthorized)?;

        Ok(VerifiedUser {
            uid: decoded.claims.sub,
            email: decoded.claims.email,
        })
    }
}
