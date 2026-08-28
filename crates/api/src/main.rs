use std::sync::Arc;

use terrarium_api::{app, spawn_tick_loop, AppState, Config, Db, IdentityService, WorldBackend, WorldHost};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("terrarium_api=info".parse().unwrap()))
        .init();

    let config = Config::from_env();
    let db = Arc::new(Db::open(&config.database_path).expect("open database"));
    let local_world = Arc::new(WorldHost::new());
    let world = WorldBackend::from_config(
        config.host_url.clone(),
        config.host_token.clone(),
        local_world.clone(),
    );
    if !world.is_remote() {
        spawn_tick_loop(local_world);
    }

    let state = AppState {
        db,
        world,
        config: config.clone(),
        identity: Arc::new(IdentityService::new(&config)),
    };

    let router = app(state);
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("bind listen addr");
    tracing::info!(
        env = config.env.name(),
        free_mint = config.env.allows_free_mint(),
        firebase = config.firebase_project_id.is_some(),
        dev_auth = config.dev_auth,
        host_remote = config.host_url.is_some(),
        addr = %config.listen_addr,
        "terrarium-api listening"
    );
    axum::serve(listener, router).await.expect("serve");
}
