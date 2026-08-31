use std::sync::Arc;

use sqlx::SqlitePool;

use crate::config::Config;
use crate::engine::WorldEngine;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
    pub engine: Arc<WorldEngine>,
}
