use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension,
};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod auth;
mod config;
mod engine;
mod ws;

use auth::FirebaseUser;
use config::Config;
use engine::{spawn_tick_loop, WorldEngine, WorldMessage};
use terrarium_kernel::{compile_wat, vm::Creature, SimConfig, CORPSE_ENERGY, WatError};

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
        .layer(middleware::from_fn(dev_only));

    let protected = Router::new()
        .route("/me", get(me))
        .route("/faucet", post(faucet))
        .route("/deploy", post(deploy))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_firebase_user,
        ));

    let v1 = Router::new()
        .route("/world", get(world))
        .route("/world/ws", get(ws::world_ws))
        .merge(protected)
        .merge(dev);

    let routes = Router::new()
        .route("/health", get(health))
        .nest("/v1", v1)
        .with_state(state);

    // /api/* for Firebase Hosting → Cloud Run rewrites; bare paths for local dev.
    Router::new()
        .merge(routes.clone())
        .nest("/api", routes)
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
    Extension(user): Extension<FirebaseUser>,
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
    Extension(user): Extension<FirebaseUser>,
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

async fn deploy(
    State(state): State<AppState>,
    Extension(user): Extension<FirebaseUser>,
    Json(body): Json<DeployRequest>,
) -> impl IntoResponse {
    match deploy_creature(&state, &user.uid, body.x, body.y, &body.code, body.energy).await {
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
) -> Result<DeployResponse, DeployError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(DeployError::InvalidProgram("program is required".into()));
    }
    if code.len() > MAX_CREATURE_CODE_LEN {
        return Err(DeployError::InvalidProgram("program too long".into()));
    }
    let min_extra = state.config.deploy_cost;
    if extra < min_extra {
        return Err(DeployError::InvalidEnergy(format!(
            "extra energy must be at least {min_extra}"
        )));
    }
    let wasm = compile_wat(code).map_err(|err| {
        DeployError::InvalidProgram(match err {
            WatError::Empty => "program is required".into(),
            WatError::Parse(msg) => msg,
        })
    })?;

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

    sqlx::query(
        "INSERT INTO creatures (id, owner_uid, x, y, energy, code, bytecode, born_tick, pc, stack) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, x'')",
    )
    .bind(&id)
    .bind(uid)
    .bind(x)
    .bind(y)
    .bind(energy)
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
