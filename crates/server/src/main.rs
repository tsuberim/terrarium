use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Extension,
};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod api_keys;
mod auth;
mod cloud_run;
mod config;
mod docs;
mod engine;
mod ws;

use auth::AuthenticatedUser;
use config::Config;
use engine::{spawn_tick_loop, WorldEngine, WorldMessage};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use terrarium_kernel::{
    compile_wat, host, vm::Creature, SimConfig, CORPSE_ENERGY, WatError,
};

#[derive(Clone)]
pub struct AppState {
    db: SqlitePool,
    config: Arc<Config>,
    engine: Arc<WorldEngine>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    tick_hz: u32,
}

#[derive(Serialize)]
struct MeResponse {
    uid: String,
    credits: i64,
}

#[derive(Deserialize)]
struct FaucetRequest {
    amount: i64,
}

#[derive(Serialize)]
struct FaucetResponse {
    credits: i64,
}

#[derive(Serialize)]
struct WorldResponse {
    deploy_cost: i64,
    corpse_energy: i64,
    creatures: Vec<engine::CreaturePublic>,
    tiles: Vec<engine::TilePublic>,
}

#[derive(Deserialize)]
struct DeployRequest {
    x: i64,
    y: i64,
    code: String,
    /// Extra energy N beyond the corpse floor; costs N credits.
    energy: i64,
    /// Precompiled WASM (standard base64). Skips WAT compile; `code` is stored as label only.
    #[serde(default)]
    wasm_b64: Option<String>,
}

#[derive(Serialize)]
struct ClearWorldResponse {
    ok: bool,
}

#[derive(Serialize)]
struct ServerPowerStatusResponse {
    power_control_available: bool,
    is_admin: bool,
    min_instances: Option<i32>,
    enabled: Option<bool>,
}

#[derive(Deserialize)]
struct ServerPowerRequest {
    enabled: bool,
}

#[derive(Serialize)]
struct ServerPowerResponse {
    ok: bool,
    enabled: bool,
    min_instances: i32,
}

#[derive(Serialize)]
struct DeployResponse {
    id: String,
    x: i64,
    y: i64,
    energy: i64,
    credits: i64,
}

const MAX_CREATURE_CODE_LEN: usize = 32_768;
const MAX_WASM_BYTES: usize = 64 * 1024;
const MAX_WASM_B64_LEN: usize = 96 * 1024;

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
    let tables: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'")
        .fetch_one(&db)
        .await?;
    if tables.0 == 0 {
        anyhow::bail!("database migration failed: accounts table missing");
    }
    tracing::info!(database_url = %redact_db_url(&config.database_url), "database ready");

    let engine = WorldEngine::bootstrap(db.clone(), &config).await?;
    spawn_tick_loop(engine.clone());

    let state = AppState {
        db,
        config: config.clone(),
        engine,
    };

    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!(addr = %config.listen_addr, "terrarium-server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_app(state: AppState) -> Router {
    let dev = Router::new()
        .route("/dev/sim-config", get(get_sim_config).patch(patch_sim_config))
        .route("/dev/clear-world", post(clear_world))
        .layer(middleware::from_fn(dev_only));

    let admin = Router::new()
        .route("/admin/server-power", get(get_server_power).post(set_server_power))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_firebase_user,
        ));

    let api_keys = Router::new()
        .route("/api-keys", get(list_api_keys).post(mint_api_key))
        .route("/api-keys/{id}", delete(revoke_api_key))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_firebase_user,
        ));

    let protected = Router::new()
        .route("/me", get(me))
        .route("/faucet", post(faucet))
        .route("/deploy", post(deploy))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_user,
        ));

    let v1 = Router::new()
        .route("/world", get(world))
        .route("/world/ws", get(ws::world_ws))
        .merge(protected)
        .merge(api_keys)
        .merge(admin)
        .merge(dev);

    let routes = Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(docs::openapi_raw))
        .nest("/v1", v1)
        .with_state(state);

    // /api/* for Firebase Hosting → Cloud Run rewrites; bare paths for local dev.
    Router::new()
        .merge(routes.clone())
        .route("/docs", get(docs::scalar))
        .nest("/api", routes)
        .route("/api/docs", get(docs::scalar_api))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        tick_hz: terrarium_kernel::TICK_HZ,
    })
}

async fn dev_only(req: Request, next: Next) -> Response {
    let dev = std::env::var("DEV_MODE")
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
        .unwrap_or(cfg!(debug_assertions));
    if dev {
        next.run(req).await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn get_sim_config(State(state): State<AppState>) -> Json<SimConfig> {
    Json(state.engine.sim_config())
}

async fn patch_sim_config(
    State(state): State<AppState>,
    Json(body): Json<SimConfig>,
) -> Json<SimConfig> {
    state.engine.set_sim_config(body.clone());
    Json(body)
}

async fn me(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    match account_credits(&state.db, &user.uid).await {
        Ok(credits) => Json(MeResponse {
            uid: user.uid,
            credits,
        })
        .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "failed to load account");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn faucet(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<FaucetRequest>,
) -> impl IntoResponse {
    if !state.config.faucet_enabled {
        return StatusCode::FORBIDDEN.into_response();
    }
    if body.amount <= 0 || body.amount > state.config.faucet_max {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("amount must be 1..={}", state.config.faucet_max)
            })),
        )
            .into_response();
    }

    match add_credits(&state.db, &user.uid, body.amount).await {
        Ok(credits) => Json(FaucetResponse { credits }).into_response(),
        Err(err) => {
            tracing::error!(error = %err, "faucet failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn world(State(state): State<AppState>) -> impl IntoResponse {
    match state.engine.snapshot() {
        WorldMessage::Snapshot {
            deploy_cost,
            corpse_energy,
            creatures,
            tiles,
            ..
        } => Json(WorldResponse {
            deploy_cost,
            corpse_energy,
            creatures,
            tiles,
        })
        .into_response(),
        WorldMessage::Delta { .. } => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn clear_world(State(state): State<AppState>) -> impl IntoResponse {
    match state.engine.clear_world().await {
        Ok(()) => Json(ClearWorldResponse { ok: true }).into_response(),
        Err(err) => {
            tracing::error!(error = %err, "clear world failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn is_admin(config: &Config, uid: &str) -> bool {
    config.admin_uids.iter().any(|id| id == uid)
}

async fn get_server_power(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    let admin = is_admin(&state.config, &user.uid);
    let Some(power) = cloud_run::CloudRunPower::from_env() else {
        return Json(ServerPowerStatusResponse {
            power_control_available: false,
            is_admin: admin,
            min_instances: None,
            enabled: None,
        })
        .into_response();
    };

    match power.min_instances().await {
        Ok(min) => Json(ServerPowerStatusResponse {
            power_control_available: true,
            is_admin: admin,
            enabled: Some(min > 0),
            min_instances: Some(min),
        })
        .into_response(),
        Err(err) => {
            tracing::warn!(error = %err, "server power status failed");
            Json(ServerPowerStatusResponse {
                power_control_available: true,
                is_admin: admin,
                min_instances: None,
                enabled: None,
            })
            .into_response()
        }
    }
}

async fn set_server_power(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<ServerPowerRequest>,
) -> impl IntoResponse {
    if !is_admin(&state.config, &user.uid) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some(power) = cloud_run::CloudRunPower::from_env() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "server power control not configured" })),
        )
            .into_response();
    };

    let min = if body.enabled {
        state.config.server_min_instances_on.max(0)
    } else {
        0
    };

    match power.set_min_instances(min).await {
        Ok(()) => Json(ServerPowerResponse {
            ok: true,
            enabled: body.enabled,
            min_instances: min,
        })
        .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "set server power failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err.to_string() })),
            )
                .into_response()
        }
    }
}

async fn deploy(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<DeployRequest>,
) -> impl IntoResponse {
    match deploy_creature(
        &state,
        &user.uid,
        body.x,
        body.y,
        &body.code,
        body.energy,
        body.wasm_b64.as_deref(),
    )
    .await {
        Ok(res) => Json(res).into_response(),
        Err(DeployError::InsufficientCredits) => (
            StatusCode::PAYMENT_REQUIRED,
            Json(serde_json::json!({ "error": "insufficient credits" })),
        )
            .into_response(),
        Err(DeployError::Occupied) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "cell occupied" })),
        )
            .into_response(),
        Err(DeployError::InvalidEnergy(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
        Err(DeployError::InvalidProgram(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
        )
            .into_response(),
        Err(DeployError::Internal(err)) => {
            tracing::error!(error = %err, "deploy failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
struct MintApiKeyRequest {
    #[serde(default)]
    name: String,
}

#[derive(Serialize)]
struct ApiKeyListResponse {
    keys: Vec<api_keys::ApiKeyPublic>,
}

async fn list_api_keys(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    match api_keys::list_keys(&state.db, &user.uid).await {
        Ok(keys) => Json(ApiKeyListResponse { keys }).into_response(),
        Err(err) => {
            tracing::error!(error = %err, "list api keys failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn mint_api_key(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<MintApiKeyRequest>,
) -> impl IntoResponse {
    match api_keys::mint_key(&state.db, &user.uid, &body.name).await {
        Ok(res) => (StatusCode::CREATED, Json(res)).into_response(),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("max") {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })))
                    .into_response();
            }
            tracing::error!(error = %err, "mint api key failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match api_keys::revoke_key(&state.db, &user.uid, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            tracing::error!(error = %err, "revoke api key failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn require_user(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let token = match bearer_token(req.headers()) {
        Some(token) => token,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let uid = if token.starts_with("tr_") {
        match api_keys::verify_key(&state.db, token).await {
            Some(uid) => uid,
            None => return StatusCode::UNAUTHORIZED.into_response(),
        }
    } else {
        match auth::verify_id_token(&state.config.firebase_project_id, token).await {
            Ok(user) => user.uid,
            Err(err) => {
                tracing::warn!(error = %err, "auth failed");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
    };

    req.extensions_mut().insert(AuthenticatedUser { uid });
    next.run(req).await
}

async fn require_firebase_user(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let token = match bearer_token(req.headers()) {
        Some(token) => token,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    match auth::verify_id_token(&state.config.firebase_project_id, token).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(err) => {
            tracing::warn!(error = %err, "firebase auth failed");
            StatusCode::UNAUTHORIZED.into_response()
        }
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

async fn account_credits(db: &SqlitePool, uid: &str) -> anyhow::Result<i64> {
    ensure_account(db, uid).await?;
    let credits = sqlx::query_scalar::<_, i64>("SELECT credits FROM accounts WHERE firebase_uid = ?")
        .bind(uid)
        .fetch_one(db)
        .await?;
    Ok(credits)
}

async fn add_credits(db: &SqlitePool, uid: &str, amount: i64) -> anyhow::Result<i64> {
    ensure_account(db, uid).await?;
    sqlx::query("UPDATE accounts SET credits = credits + ? WHERE firebase_uid = ?")
        .bind(amount)
        .bind(uid)
        .execute(db)
        .await?;
    account_credits(db, uid).await
}

async fn ensure_account(db: &SqlitePool, uid: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO accounts (firebase_uid, credits) VALUES (?, 0) ON CONFLICT DO NOTHING",
    )
    .bind(uid)
    .execute(db)
    .await?;
    Ok(())
}

enum DeployError {
    InsufficientCredits,
    Occupied,
    InvalidEnergy(String),
    InvalidProgram(String),
    Internal(anyhow::Error),
}

fn deploy_internal(err: impl Into<anyhow::Error>) -> DeployError {
    DeployError::Internal(err.into())
}

async fn deploy_creature(
    state: &AppState,
    uid: &str,
    x: i64,
    y: i64,
    code: &str,
    extra: i64,
    wasm_b64: Option<&str>,
) -> Result<DeployResponse, DeployError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(DeployError::InvalidProgram("program is required".into()));
    }

    let wasm = if let Some(b64) = wasm_b64 {
        if b64.len() > MAX_WASM_B64_LEN {
            return Err(DeployError::InvalidProgram("wasm too long".into()));
        }
        if code.len() > 4096 {
            return Err(DeployError::InvalidProgram("program too long".into()));
        }
        let bytes = STANDARD.decode(b64.trim()).map_err(|_| {
            DeployError::InvalidProgram("invalid wasm encoding".into())
        })?;
        if bytes.is_empty() || bytes.len() > MAX_WASM_BYTES {
            return Err(DeployError::InvalidProgram("invalid wasm size".into()));
        }
        if host::load_module(host::wasm_engine(), &bytes).is_none() {
            return Err(DeployError::InvalidProgram("invalid wasm module".into()));
        }
        bytes
    } else {
        if code.len() > MAX_CREATURE_CODE_LEN {
            return Err(DeployError::InvalidProgram("program too long".into()));
        }
        compile_wat(code).map_err(|err| {
            DeployError::InvalidProgram(match err {
                WatError::Empty => "program is required".into(),
                WatError::Parse(msg) => msg,
            })
        })?
    };

    let min_extra = state.config.deploy_cost;
    if extra < min_extra {
        return Err(DeployError::InvalidEnergy(format!(
            "extra energy must be at least {min_extra}"
        )));
    }

    let cost = extra;
    let id = Uuid::new_v4().to_string();
    let energy = CORPSE_ENERGY + extra;

    if !state.engine.is_deployable(x, y) {
        return Err(DeployError::Occupied);
    }

    let mut tx = state.db.begin().await.map_err(deploy_internal)?;

    ensure_account(&state.db, uid)
        .await
        .map_err(deploy_internal)?;

    let credits: i64 = sqlx::query_scalar("SELECT credits FROM accounts WHERE firebase_uid = ?")
        .bind(uid)
        .fetch_one(&mut *tx)
        .await
        .map_err(deploy_internal)?;

    if credits < cost {
        return Err(DeployError::InsufficientCredits);
    }

    sqlx::query("UPDATE accounts SET credits = credits - ? WHERE firebase_uid = ?")
        .bind(cost)
        .bind(uid)
        .execute(&mut *tx)
        .await
        .map_err(deploy_internal)?;

    let born_tick = state.engine.current_tick() as i64;
    let sim = state.engine.sim_config();

    sqlx::query(
        "INSERT INTO creatures (id, owner_uid, x, y, energy, health, max_health, code, bytecode, born_tick, pc, stack) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, x'')",
    )
    .bind(&id)
    .bind(uid)
    .bind(x)
    .bind(y)
    .bind(energy)
    .bind(sim.max_health as i64)
    .bind(sim.max_health as i64)
    .bind(code)
    .bind(&wasm)
    .bind(born_tick)
    .execute(&mut *tx)
    .await
    .map_err(|err| {
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.is_unique_violation() {
                return DeployError::Occupied;
            }
        }
        DeployError::Internal(err.into())
    })?;

    tx.commit().await.map_err(deploy_internal)?;

    state
        .engine
        .insert_creature(Creature {
            id: id.clone(),
            owner_uid: uid.to_string(),
            x: x as i32,
            y: y as i32,
            energy,
            health: sim.max_health,
            max_health: sim.max_health,
            parent_id: None,
            wasm,
            code: code.to_string(),
            alive: true,
            inbox: vec![],
            death_reason: None,
            born_tick: born_tick as u64,
        })
        .map_err(deploy_internal)?;

    let credits = account_credits(&state.db, uid)
        .await
        .map_err(deploy_internal)?;

    Ok(DeployResponse {
        id,
        x,
        y,
        energy,
        credits,
    })
}
