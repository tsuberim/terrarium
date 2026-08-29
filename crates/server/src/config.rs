use std::env;

use anyhow::{Context, bail};

#[derive(Clone, Debug)]
pub struct Config {
    pub listen_addr: String,
    pub database_url: String,
    pub firebase_project_id: String,
    pub faucet_enabled: bool,
    pub faucet_max: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let firebase_project_id =
            env::var("FIREBASE_PROJECT_ID").context("FIREBASE_PROJECT_ID is required")?;

        let port = env::var("PORT").unwrap_or_else(|_| "8080".into());
        let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| format!("0.0.0.0:{port}"));

        Ok(Self {
            listen_addr,
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/terrarium.db".into()),
            firebase_project_id,
            faucet_enabled: env_bool("FAUCET_ENABLED", true),
            faucet_max: env::var("FAUCET_MAX")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(10_000),
        })
    }
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(default)
}

pub fn ensure_parent_dir(database_url: &str) -> anyhow::Result<()> {
    if database_url.contains(":memory:") {
        return Ok(());
    }
    let path = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .context("DATABASE_URL must use sqlite: or sqlite:// prefix")?;
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    } else {
        bail!("invalid DATABASE_URL");
    }
    Ok(())
}
