//! Native Terrarium host binary — authoritative world + WebSocket camera feed.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use terrarium_host::{spawn_tick_loop, wrap_snapshot, WorldHost};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    world: Arc<WorldHost>,
    snapshots: broadcast::Sender<String>,
    env: String,
    internal_token: Option<String>,
    skin_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMsg {
    SetProgram { cell_id: u64, source: String },
    Reset,
}

#[derive(Debug, Deserialize)]
struct InternalSpawnRequest {
    mass: u64,
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default)]
    program: Option<String>,
}

#[derive(serde::Serialize)]
struct InternalSpawnResponse {
    cell_id: u64,
    world_spawned_mass: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let env = std::env::var("TERRARIUM_ENV").unwrap_or_else(|_| "local".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let internal_token = std::env::var("TERRARIUM_HOST_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    let skin_dir = std::env::var("SKIN_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            let dir = skin_dir_from_cwd();
            if dir.join("index.html").is_file() {
                Some(dir)
            } else {
                None
            }
        });

    let world = Arc::new(WorldHost::new());
    let (snapshots, _) = broadcast::channel::<String>(16);
    spawn_tick_loop(world.clone(), snapshots.clone());

    let state = AppState {
        world,
        snapshots,
        env: env.clone(),
        internal_token,
        skin_dir: skin_dir.clone(),
    };

    let mut app = Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route("/ws", get(ws_handler))
        .route("/internal/spawn", post(internal_spawn))
        .route("/internal/snapshot", get(internal_snapshot));

    if let Some(ref dir) = skin_dir {
        let static_files = ServeDir::new(dir)
            .append_index_html_on_directories(true)
            .precompressed_gzip();
        app = app
            .route("/", get(serve_index))
            .route("/index.html", get(serve_index))
            .fallback_service(static_files);
        info!(skin = %dir.display(), "serving skin static files (local dev)");
    }

    let app = app
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, env = %env, "terrarium-host listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve");
}

fn skin_dir_from_cwd() -> PathBuf {
    for c in [
        PathBuf::from("apps/skin"),
        PathBuf::from("../apps/skin"),
        PathBuf::from("../../apps/skin"),
    ] {
        if c.join("index.html").is_file() {
            return c;
        }
    }
    PathBuf::from("apps/skin")
}

async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let Some(ref skin_dir) = state.skin_dir else {
        return (StatusCode::NOT_FOUND, "skin not configured").into_response();
    };
    match read_index_with_env(skin_dir, &state.env) {
        Ok(body) => (
            StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            )],
            body,
        )
            .into_response(),
        Err(err) => {
            warn!(%err, "failed to read index.html");
            (StatusCode::INTERNAL_SERVER_ERROR, "missing skin index.html").into_response()
        }
    }
}

fn read_index_with_env(skin_dir: &Path, env: &str) -> std::io::Result<String> {
    let raw = std::fs::read_to_string(skin_dir.join("index.html"))?;
    let needle = r#"content="staging""#;
    let replacement = format!(r#"content="{env}""#);
    Ok(raw.replace(needle, &replacement))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| client_session(socket, state))
}

async fn client_session(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();
    let mut rx = state.snapshots.subscribe();

    let hello = serde_json::json!({
        "type": "hello",
        "env": state.env,
    })
    .to_string();
    if sink.send(Message::Text(hello.into())).await.is_err() {
        return;
    }

    if let Ok(json) = state.world.snapshot_json() {
        let snap = wrap_snapshot(&json);
        if sink.send(Message::Text(snap.into())).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            pushed = rx.recv() => {
                match pushed {
                    Ok(json) => {
                        if sink.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            incoming = stream.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if let Some(reply) = handle_client_text(&state, text.as_ref()).await {
                            if sink.send(Message::Text(reply.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Ping(p))) => {
                        if sink.send(Message::Pong(p)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

async fn handle_client_text(state: &AppState, text: &str) -> Option<String> {
    let msg: ClientMsg = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(err) => {
            return Some(
                serde_json::json!({
                    "type": "error",
                    "message": format!("bad client message: {err}"),
                })
                .to_string(),
            );
        }
    };

    match msg {
        ClientMsg::SetProgram { cell_id, source } => match state.world.set_program(cell_id, &source) {
            Ok(()) => Some(
                serde_json::json!({
                    "type": "ok",
                    "message": format!("running on cell {cell_id}."),
                })
                .to_string(),
            ),
            Err(err) => Some(
                serde_json::json!({
                    "type": "error",
                    "message": err,
                })
                .to_string(),
            ),
        },
        ClientMsg::Reset => {
            state.world.reset();
            if let Ok(json) = state.world.snapshot_json() {
                let _ = state.snapshots.send(wrap_snapshot(&json));
            }
            Some(
                serde_json::json!({
                    "type": "ok",
                    "message": "world reset.",
                })
                .to_string(),
            )
        }
    }
}

fn authorize_internal(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    let Some(expected) = state.internal_token.as_ref() else {
        return Ok(());
    };
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token == expected {
        Ok(())
    } else {
        Err((StatusCode::UNAUTHORIZED, "invalid host token".into()))
    }
}

async fn internal_spawn(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<InternalSpawnRequest>,
) -> Result<Json<InternalSpawnResponse>, (StatusCode, String)> {
    authorize_internal(&state, &headers)?;
    if body.mass == 0 {
        return Err((StatusCode::BAD_REQUEST, "mass must be greater than zero".into()));
    }
    let cell_id = state
        .world
        .spawn_cell(body.mass, body.x, body.y, body.program.as_deref())
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let spawned_mass = state
        .world
        .snapshot_json()
        .ok()
        .and_then(|j| serde_json::from_str::<serde_json::Value>(&j).ok())
        .and_then(|v| v.get("spawned_mass").and_then(|m| m.as_u64()))
        .unwrap_or(body.mass);
    Ok(Json(InternalSpawnResponse {
        cell_id,
        world_spawned_mass: spawned_mass,
    }))
}

async fn internal_snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<String, (StatusCode, String)> {
    authorize_internal(&state, &headers)?;
    state
        .world
        .snapshot_json()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("ctrl_c handler install");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("sigterm handler install")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let world = Arc::new(WorldHost::new());
        let (snapshots, _) = broadcast::channel(4);
        AppState {
            world,
            snapshots,
            env: "local".into(),
            internal_token: Some("test-token".into()),
            skin_dir: None,
        }
    }

    #[tokio::test]
    async fn internal_spawn_requires_token() {
        let state = test_state();
        let app = Router::new()
            .route("/internal/spawn", post(internal_spawn))
            .with_state(state);

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/internal/spawn")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"mass":100}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn internal_spawn_with_token() {
        let state = test_state();
        let app = Router::new()
            .route("/internal/spawn", post(internal_spawn))
            .with_state(state);

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/internal/spawn")
                    .header("Content-Type", "application/json")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::from(r#"{"mass":100,"x":0,"y":0}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["cell_id"].as_u64().unwrap() > 0, true);
    }
}
