use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use terrarium_sim::SimConfig;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::accounts::{account_credits, add_credits};
use crate::api_keys;
use crate::auth::AuthenticatedUser;
use crate::compile_client;
use crate::control_ws;
use crate::deploy::{deploy_creature, DeployError};
use crate::docs;
use crate::middleware::{dev_only, require_api_key, require_firebase_user, require_user};
use crate::sandbox;
use crate::state::AppState;
use crate::wire::{CreaturePublic, TilePublic};
use crate::ws;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    tick_hz: u32,
}

#[derive(Serialize)]
struct MeResponse {
    uid: String,
    credits: i64,
    #[serde(with = "crate::wire::serde_u64")]
    account_creature_id: u64,
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
    creatures: Vec<CreaturePublic>,
    tiles: Vec<TilePublic>,
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

#[derive(Deserialize)]
struct CompileBody {
    language: String,
    source: String,
    #[serde(default)]
    tests: String,
}

#[derive(Deserialize)]
struct SandboxRunBody {
    wasm_b64: String,
    test: terrarium_sim::TestSpec,
}

#[derive(Serialize)]
struct ClearWorldResponse {
    ok: bool,
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

pub fn build_app(state: AppState) -> Router {
    let dev = Router::new()
        .route(
            "/dev/sim-config",
            get(get_sim_config).patch(patch_sim_config),
        )
        .route("/dev/clear-world", post(clear_world))
        .layer(middleware::from_fn(dev_only));

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
        .route("/compile", post(compile))
        .route("/sandbox/run", post(sandbox_run))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_user));

    let control = Router::new()
        .route("/control/ws", get(control_ws::control_ws))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));

    let v1 = Router::new()
        .route("/world", get(world))
        .route("/world/ws", get(ws::world_ws))
        .merge(protected)
        .merge(api_keys)
        .merge(control)
        .merge(dev);

    let routes = Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(docs::openapi_raw))
        .nest("/v1", v1)
        .with_state(state);

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
        tick_hz: terrarium_sim::TICK_HZ,
    })
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
        Ok(credits) => match crate::accounts::account_creature_id(&state.db, &user.uid).await {
            Ok(account_creature_id) => Json(MeResponse {
                uid: user.uid,
                credits,
                account_creature_id,
            })
            .into_response(),
            Err(err) => {
                tracing::error!(error = %err, "failed to load account_creature_id");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
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
    let (deploy_cost, corpse_energy, creatures, tiles) = state.engine.world_http();
    Json(WorldResponse {
        deploy_cost,
        corpse_energy,
        creatures,
        tiles,
    })
    .into_response()
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
    .await
    {
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

async fn compile(
    State(state): State<AppState>,
    Json(body): Json<CompileBody>,
) -> impl IntoResponse {
    let Some(worker_url) = state.config.compile_worker_url.as_deref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "compile worker not configured" })),
        )
            .into_response();
    };

    match compile_client::compile_creature(worker_url, &body.language, &body.source, &body.tests)
        .await
    {
        Ok(res) => {
            if res.ok {
                Json(res).into_response()
            } else {
                (StatusCode::BAD_REQUEST, Json(res)).into_response()
            }
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": err })),
        )
            .into_response(),
    }
}

async fn sandbox_run(
    State(state): State<AppState>,
    Json(body): Json<SandboxRunBody>,
) -> impl IntoResponse {
    let wasm = match sandbox::decode_wasm(&body.wasm_b64) {
        Ok(bytes) => bytes,
        Err(msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": msg })),
            )
                .into_response();
        }
    };

    let mut config = state.engine.sim_config();
    config.max_active_food = 0;
    config.food_spawn_interval = u64::MAX;
    let result = sandbox::run_creature_sandbox(&wasm, body.test, Some(config));
    Json(result).into_response()
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
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": msg })),
                )
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
    Path(id): Path<String>,
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
