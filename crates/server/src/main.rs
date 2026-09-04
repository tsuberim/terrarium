use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod accounts;
mod api_keys;
mod auth;
mod compile_client;
mod config;
mod control;
mod control_ws;
mod deploy;
mod docs;
mod engine;
mod ids;
mod middleware;
mod persist;
mod routes;
mod sandbox;
mod seed;
mod state;
mod tick_loop;
mod wire;
mod ws;

use config::Config;
use control::ControlRegistry;
use engine::WorldEngine;
use routes::build_app;
use state::AppState;
use tick_loop::spawn_tick_loop;

fn redact_db_url(url: &str) -> String {
    url.split('?').next().unwrap_or(url).to_string()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::from_env()?);
    config::ensure_parent_dir(&config.database_url)?;
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!().run(&db).await?;
    let tables: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'")
            .fetch_one(&db)
            .await?;
    if tables.0 == 0 {
        anyhow::bail!("database migration failed: accounts table missing");
    }
    tracing::info!(database_url = %redact_db_url(&config.database_url), "database ready");

    let control = ControlRegistry::new();
    let engine = WorldEngine::bootstrap(db.clone(), &config).await?;
    engine.attach_control(control.clone());
    spawn_tick_loop(engine.clone(), control.clone());

    let state = AppState {
        db,
        config: config.clone(),
        engine,
        control,
    };

    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!(addr = %config.listen_addr, "terrarium-server listening");
    axum::serve(listener, app).await?;
    Ok(())
}
