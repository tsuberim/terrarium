//! Scale Cloud Run min instances via the Run Admin API (GCP metadata token).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CloudRunPower {
    project_id: String,
    region: String,
    service: String,
    client: reqwest::Client,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scaling {
    min_instance_count: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PatchBody {
    scaling: Scaling,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceResponse {
    scaling: Option<Scaling>,
}

#[derive(Deserialize)]
struct MetadataToken {
    access_token: String,
}

impl CloudRunPower {
    pub fn from_env() -> Option<Self> {
        let project_id = std::env::var("GCP_PROJECT_ID").ok()?;
        let region = std::env::var("GCP_REGION").unwrap_or_else(|_| "us-central1".into());
        let service =
            std::env::var("CLOUD_RUN_SERVICE").unwrap_or_else(|_| "terrarium-server".into());
        Some(Self {
            project_id,
            region,
            service,
            client: reqwest::Client::new(),
        })
    }

    fn service_url(&self) -> String {
        format!(
            "https://run.googleapis.com/v2/projects/{}/locations/{}/services/{}",
            self.project_id, self.region, self.service
        )
    }

    async fn access_token(&self) -> Result<String> {
        let resp = self
            .client
            .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
            .header("Metadata-Flavor", "Google")
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
            .context("GCP metadata token unavailable (not running on Cloud Run?)")?;
        if !resp.status().is_success() {
            anyhow::bail!("metadata token HTTP {}", resp.status());
        }
        let body: MetadataToken = resp.json().await.context("metadata token JSON")?;
        Ok(body.access_token)
    }

    pub async fn min_instances(&self) -> Result<i32> {
        let token = self.access_token().await?;
        let resp = self
            .client
            .get(self.service_url())
            .bearer_auth(token)
            .send()
            .await
            .context("run.services.get")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("run.services.get {status}: {text}");
        }
        let body: ServiceResponse = resp.json().await.context("run.services.get JSON")?;
        Ok(body
            .scaling
            .map(|s| s.min_instance_count)
            .unwrap_or(0))
    }

    pub async fn set_min_instances(&self, min: i32) -> Result<()> {
        let token = self.access_token().await?;
        let url = format!(
            "{}?updateMask=scaling.minInstanceCount",
            self.service_url()
        );
        let resp = self
            .client
            .patch(url)
            .bearer_auth(token)
            .json(&PatchBody {
                scaling: Scaling {
                    min_instance_count: min,
                },
            })
            .send()
            .await
            .context("run.services.patch")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("run.services.patch {status}: {text}");
        }
        Ok(())
    }
}
