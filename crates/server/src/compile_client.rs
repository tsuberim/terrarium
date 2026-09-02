use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileRequest {
    pub language: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_b64: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

pub async fn compile_creature(
    worker_url: &str,
    language: &str,
    source: &str,
) -> Result<CompileResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/compile", worker_url.trim_end_matches('/'));
    let body = CompileRequest {
        language: language.to_string(),
        source: source.to_string(),
    };

    let mut req = client.post(&url).json(&body);
    if let Some(token) = gcp_identity_token(worker_url).await? {
        req = req.bearer_auth(token);
    }

    let res = req.send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    let parsed: CompileResponse = serde_json::from_str(&text).map_err(|_| text.clone())?;
    if status.is_success() {
        Ok(parsed)
    } else {
        Err(parsed
            .diagnostics
            .first()
            .map(|d| d.message.clone())
            .unwrap_or_else(|| text))
    }
}

async fn gcp_identity_token(audience: &str) -> Result<Option<String>, String> {
    if std::env::var("COMPILE_WORKER_SECRET").is_ok() {
        return Ok(std::env::var("COMPILE_WORKER_SECRET").ok());
    }
    if audience.contains("localhost") || audience.contains("127.0.0.1") {
        return Ok(None);
    }

    let client = reqwest::Client::new();
    let meta_url = format!(
        "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity?audience={}",
        urlencoding::encode(audience)
    );
    let res = client
        .get(&meta_url)
        .header("Metadata-Flavor", "Google")
        .send()
        .await
        .map_err(|e| format!("metadata token fetch failed: {e}"))?;
    if !res.status().is_success() {
        return Err("failed to obtain GCP identity token for compile worker".into());
    }
    Ok(Some(res.text().await.map_err(|e| e.to_string())?))
}
