use std::sync::Arc;

use crate::config::Config;
use crate::db::Db;
use crate::error::{ApiError, ApiResult};
use crate::firebase::{FirebaseVerifier, VerifiedUser};

pub struct IdentityService {
    firebase: Option<Arc<FirebaseVerifier>>,
    dev_auth: bool,
}

impl IdentityService {
    pub fn new(config: &Config) -> Self {
        let firebase = config.firebase_project_id.as_ref().map(|id| {
            Arc::new(FirebaseVerifier::new(id.clone()))
        });
        Self {
            firebase,
            dev_auth: config.dev_auth,
        }
    }

    pub fn firebase_configured(&self) -> bool {
        self.firebase.is_some()
    }

    pub fn dev_auth_enabled(&self) -> bool {
        self.dev_auth
    }

    pub async fn authenticate_human(&self, bearer: &str, db: &Db) -> ApiResult<String> {
        if bearer.starts_with("trm_sess_") {
            if !self.dev_auth {
                return Err(ApiError::Unauthorized);
            }
            return db.account_for_session(bearer);
        }

        if bearer.starts_with("trm_") {
            return Err(ApiError::Unauthorized);
        }

        let firebase = self.firebase.as_ref().ok_or(ApiError::Unauthorized)?;
        let VerifiedUser { uid, .. } = firebase.verify(bearer).await?;
        db.ensure_account(&uid)?;
        Ok(uid)
    }
}
