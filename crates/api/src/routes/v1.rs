use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::auth::bearer_from_header;
use crate::error::{ApiError, ApiResult};
use crate::AppState;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn create_account(
    State(state): State<AppState>,
) -> ApiResult<Json<CreateAccountResponse>> {
    if state.identity.firebase_configured() || !state.identity.dev_auth_enabled() {
        return Err(ApiError::Forbidden);
    }
    let (account_id, session_token) = state.db.create_dev_account()?;
    Ok(Json(CreateAccountResponse {
        account_id,
        session_token,
        environment: state.config.env.name().to_string(),
        free_mint_enabled: state.config.env.allows_free_mint(),
        auth_mode: "dev".to_string(),
    }))
}

#[derive(Serialize)]
pub struct CreateAccountResponse {
    account_id: String,
    session_token: String,
    environment: String,
    free_mint_enabled: bool,
    auth_mode: String,
}

#[derive(Deserialize)]
pub struct SpawnRequest {
    pub mass: u64,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub program: Option<String>,
}

#[derive(Serialize)]
pub struct SpawnResponse {
    spawn_id: String,
    cell_id: u64,
    mass: u64,
    x: i32,
    y: i32,
    credits_remaining: u64,
    world_spawned_mass: u64,
}

pub async fn spawn(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SpawnRequest>,
) -> ApiResult<(StatusCode, Json<SpawnResponse>)> {
    let token = bearer_from_header(headers.get("authorization").and_then(|v| v.to_str().ok()))
        .ok_or(ApiError::Unauthorized)?;
    let (account_id, scopes) = state.db.account_for_api_token(token)?;
    scopes.require_spawn()?;
    if body.mass == 0 {
        return Err(ApiError::BadRequest("mass must be greater than zero".into()));
    }
    let cell_id = state
        .world
        .spawn_cell(
        body.mass,
        body.x,
        body.y,
        body.program.as_deref(),
    )
    .await?;
    let (spawn_id, credits_remaining) = state
        .db
        .spend_and_record_spawn(&account_id, body.mass, cell_id, body.x, body.y)?;
    let world_spawned_mass = state.world.spawned_mass().await.unwrap_or(body.mass);
    Ok((
        StatusCode::CREATED,
        Json(SpawnResponse {
            spawn_id,
            cell_id,
            mass: body.mass,
            x: body.x,
            y: body.y,
            credits_remaining,
            world_spawned_mass,
        }),
    ))
}

pub async fn world_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let token = bearer_from_header(headers.get("authorization").and_then(|v| v.to_str().ok()))
        .ok_or(ApiError::Unauthorized)?;
    let (_account_id, scopes) = state.db.account_for_api_token(token)?;
    scopes.require_read()?;
    let json = state.world.snapshot_json().await?;
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|_| ApiError::Internal)?;
    Ok(Json(value))
}
