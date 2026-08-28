mod auth;
mod config;
mod db;
mod error;
mod firebase;
mod identity;
mod routes;
mod scopes;

pub use config::{Config, Environment, FirebaseWebConfig};
pub use db::{spawn_tick_loop, ApiTokenRow, Db, WorldHost};
pub use error::{ApiError, ApiResult};
pub use identity::IdentityService;
pub use scopes::TokenScopes;

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub world: Arc<WorldHost>,
    pub config: Config,
    pub identity: Arc<IdentityService>,
}

pub fn app(state: AppState) -> Router {
    let api = Router::new()
        .route("/accounts", post(routes::v1::create_account))
        .route("/spawn", post(routes::v1::spawn))
        .route("/world/snapshot", get(routes::v1::world_snapshot));

    let dashboard = Router::new()
        .route("/config", get(routes::dashboard::config))
        .route("/me", get(routes::dashboard::me))
        .route("/tokens", get(routes::dashboard::list_tokens))
        .route("/tokens", post(routes::dashboard::mint_token))
        .route("/tokens/{id}", delete(routes::dashboard::revoke_token))
        .route("/faucet", post(routes::dashboard::faucet))
        .route("/billing/checkout", post(routes::dashboard::billing_checkout));

    let dashboard_assets = ServeDir::new(state.config.dashboard_dir.clone());

    Router::new()
        .route("/health", get(routes::health))
        .nest("/v1", api)
        .nest("/dashboard/api", dashboard)
        .nest_service("/dashboard", dashboard_assets)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
