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

mod auth;
mod config;

use auth::FirebaseUser;
use config::Config;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    config: Arc<Config>,
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

    let state = AppState {
        db,
        config: config.clone(),
    };

    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    tracing::info!(addr = %config.listen_addr, "terrarium-server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_app(state: AppState) -> Router {
    let api = Router::new()
        .route("/me", get(me))
        .route("/faucet", post(faucet))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_firebase_user,
        ));

    let routes = Router::new()
        .route("/health", get(health))
        .nest("/v1", api)
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
