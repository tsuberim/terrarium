use std::sync::Arc;

use terrarium_api::{app, spawn_tick_loop, AppState, Config, Db, WorldHost};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("terrarium_api=info".parse().unwrap()))
        .init();

    let config = Config::from_env();
    let db = Arc::new(Db::open(&config.database_path).expect("open database"));
    let world = Arc::new(WorldHost::new());
    spawn_tick_loop(world.clone());

    let state = AppState {
        db,
        world,
        config: config.clone(),
    };

    let router = app(state);
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("bind listen addr");
    tracing::info!(
        env = config.env.name(),
        free_mint = config.env.allows_free_mint(),
        addr = %config.listen_addr,
        "terrarium-api listening"
    );
    axum::serve(listener, router).await.expect("serve");
}
