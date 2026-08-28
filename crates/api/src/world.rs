use std::sync::Arc;

use crate::error::{ApiError, ApiResult};

/// Where the authoritative world lives: in-process (CI/local) or remote host.
#[derive(Clone)]
pub enum WorldBackend {
    Local(Arc<super::WorldHost>),
    Remote(RemoteWorldHost),
}

#[derive(Clone)]
pub struct RemoteWorldHost {
    base_url: String,
    token: Option<String>,
}

impl RemoteWorldHost {
    pub fn new(base_url: String, token: Option<String>) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self { base_url, token }
    }

    pub async fn spawn_cell(
        &self,
        mass: u64,
        x: i32,
        y: i32,
        program: Option<&str>,
    ) -> ApiResult<u64> {
        let body = serde_json::json!({
            "mass": mass,
            "x": x,
            "y": y,
            "program": program,
        });
        let mut req = attohttpc::post(format!("{}/internal/spawn", self.base_url))
            .header("Content-Type", "application/json")
            .bytes(serde_json::to_vec(&body).map_err(|_| ApiError::Internal)?);
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let resp = req.send().map_err(|_| ApiError::Internal)?;
        if resp.status() == 401 {
            return Err(ApiError::Internal);
        }
        if !resp.status().is_success() {
            let msg = resp.text().unwrap_or_default();
            return Err(ApiError::BadRequest(msg));
        }
        let json: serde_json::Value = resp.json().map_err(|_| ApiError::Internal)?;
        json.get("cell_id")
            .and_then(|v| v.as_u64())
            .ok_or(ApiError::Internal)
    }

    pub async fn snapshot_json(&self) -> ApiResult<String> {
        let mut req = attohttpc::get(format!("{}/internal/snapshot", self.base_url));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        let resp = req.send().map_err(|_| ApiError::Internal)?;
        if !resp.status().is_success() {
            return Err(ApiError::Internal);
        }
        resp.text().map_err(|_| ApiError::Internal)
    }

    pub async fn spawned_mass(&self) -> ApiResult<u64> {
        let json = self.snapshot_json().await?;
        serde_json::from_str::<serde_json::Value>(&json)
            .ok()
            .and_then(|v| v.get("spawned_mass").and_then(|m| m.as_u64()))
            .ok_or(ApiError::Internal)
    }
}

impl WorldBackend {
    pub fn from_config(
        host_url: Option<String>,
        host_token: Option<String>,
        local: Arc<super::WorldHost>,
    ) -> Self {
        match host_url.filter(|s| !s.is_empty()) {
            Some(url) => Self::Remote(RemoteWorldHost::new(url, host_token)),
            None => Self::Local(local),
        }
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote(_))
    }

    pub async fn spawn_cell(
        &self,
        mass: u64,
        x: i32,
        y: i32,
        program: Option<&str>,
    ) -> ApiResult<u64> {
        match self {
            Self::Local(host) => host.spawn_cell(mass, x, y, program),
            Self::Remote(remote) => remote.spawn_cell(mass, x, y, program).await,
        }
    }

    pub async fn snapshot_json(&self) -> ApiResult<String> {
        match self {
            Self::Local(host) => host.snapshot_json(),
            Self::Remote(remote) => remote.snapshot_json().await,
        }
    }

    pub async fn spawned_mass(&self) -> ApiResult<u64> {
        match self {
            Self::Local(host) => {
                let json = host.snapshot_json()?;
                serde_json::from_str::<serde_json::Value>(&json)
                    .ok()
                    .and_then(|v| v.get("spawned_mass").and_then(|m| m.as_u64()))
                    .ok_or(ApiError::Internal)
            }
            Self::Remote(remote) => remote.spawned_mass().await,
        }
    }
}
