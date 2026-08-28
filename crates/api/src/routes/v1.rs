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
    let (account_id, session_token) = state.db.create_account()?;
    Ok(Json(CreateAccountResponse {
        account_id,
        session_token,
        environment: state.config.env.name().to_string(),
        free_mint_enabled: state.config.env.allows_free_mint(),
    }))
}

#[derive(Serialize)]
pub struct CreateAccountResponse {
    account_id: String,
    session_token: String,
    environment: String,
    free_mint_enabled: bool,
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
    let account_id = state.db.account_for_api_token(token)?;
    if body.mass == 0 {
        return Err(ApiError::BadRequest("mass must be greater than zero".into()));
    }
    let cell_id = state.world.spawn_cell(
        body.mass,
        body.x,
        body.y,
        body.program.as_deref(),
    )?;
    let (spawn_id, credits_remaining) = state
        .db
        .spend_and_record_spawn(&account_id, body.mass, cell_id, body.x, body.y)?;
    let world_spawned_mass = state
        .world
        .snapshot_json()
        .ok()
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .and_then(|v| v.get("spawned_mass").and_then(|m| m.as_u64()))
        .unwrap_or(body.mass);
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
    let _account = state.db.account_for_api_token(token)?;
    let json = state.world.snapshot_json()?;
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|_| ApiError::Internal)?;
    Ok(Json(value))
}
