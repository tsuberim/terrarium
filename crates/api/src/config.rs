use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Environment {
    Local,
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn parse(raw: &str) -> Self {
        match raw.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "staging" | "stage" => Self::Staging,
            "development" | "dev" => Self::Development,
            _ => Self::Local,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Development => "development",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }

    pub fn allows_free_mint(&self) -> bool {
        !matches!(self, Self::Production)
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub env: Environment,
    pub listen_addr: String,
    pub database_path: PathBuf,
    pub dashboard_dir: PathBuf,
    pub faucet_amount: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let env = Environment::parse(
            &env::var("TERRARIUM_ENV").unwrap_or_else(|_| "local".to_string()),
        );
        let listen_addr =
            env::var("TERRARIUM_LISTEN").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let database_path = env::var("TERRARIUM_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("terrarium.db"));
        let dashboard_dir = env::var("TERRARIUM_DASHBOARD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("apps/dashboard"));
        let faucet_amount = env::var("TERRARIUM_FAUCET_AMOUNT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10_000);

        Self {
            env,
            listen_addr,
            database_path,
            dashboard_dir,
            faucet_amount,
        }
    }
}
