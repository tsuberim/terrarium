//! Native Terrarium host.
//!
//! Owns one `World`, ticks it on a fixed interval without any browser open,
//! serves `apps/skin` as static files, and pushes snapshots over WebSocket.
//! The tab is a camera + program editor — not the process of record.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use terrarium_kernel::{compile_text, CellId, Mass, World};
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;
use tracing::{info, warn};

/// Match the previous in-tab cadence (~20 Hz).
const MS_PER_TICK: u64 = 50;

const DEMO_WANDER: &str = r#"# wander — fixed thrust loop (deterministic)
thrust 50 20
sleep
thrust -40 45
sleep
thrust -30 -55
sleep
thrust 55 -15
sleep
jump 0
"#;

const DEMO_CHASE: &str = r#"# chase — sense nearest body, thrust toward it
sense
jnz 0 4
sleep
jump 0
thrust_toward 70
sleep
jump 0
"#;

const DEMO_SIT: &str = r#"# sit — sleep is free
sleep
jump 0
"#;

#[derive(Clone)]
struct AppState {
    world: Arc<Mutex<World>>,
    snapshots: broadcast::Sender<String>,
    env: String,
    skin_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMsg {
    SetProgram { cell_id: u64, source: String },
    Reset,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let skin_dir = skin_dir_from_env();
    let env = std::env::var("TERRARIUM_ENV").unwrap_or_else(|_| "local".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let mut world = World::new();
    seed_world(&mut world);
    let (snapshots, _) = broadcast::channel::<String>(16);

    let state = AppState {
        world: Arc::new(Mutex::new(world)),
        snapshots,
        env: env.clone(),
        skin_dir: skin_dir.clone(),
    };

    spawn_tick_loop(state.clone());

    let static_files = ServeDir::new(&skin_dir)
        .append_index_html_on_directories(true)
        .precompressed_gzip();

    let app = Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route("/ws", get(ws_handler))
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .fallback_service(static_files)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, env = %env, skin = %skin_dir.display(), "terrarium host listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve");
}

fn skin_dir_from_env() -> PathBuf {
    if let Ok(p) = std::env::var("SKIN_DIR") {
        return PathBuf::from(p);
    }
    // Dev default: repo apps/skin relative to cwd or crate.
    let candidates = [
        PathBuf::from("apps/skin"),
        PathBuf::from("../apps/skin"),
        PathBuf::from("../../apps/skin"),
    ];
    for c in candidates {
        if c.join("index.html").is_file() {
            return c;
        }
    }
    PathBuf::from("apps/skin")
}

fn seed_world(world: &mut World) {
    *world = World::new();
    let a = world
        .spawn_cell_at(Mass::new(5000), -28000, -12000)
        .expect("spawn a");
    let b = world
        .spawn_cell_at(Mass::new(4000), 22000, 8000)
        .expect("spawn b");
    let c = world
        .spawn_cell_at(Mass::new(3500), -5000, 28000)
        .expect("spawn c");
    world
        .set_program(a, compile_text(DEMO_WANDER).expect("wander"))
        .expect("program a");
    world
        .set_program(b, compile_text(DEMO_CHASE).expect("chase"))
        .expect("program b");
    world
        .set_program(c, compile_text(DEMO_SIT).expect("sit"))
        .expect("program c");
    let dumper = world
        .spawn_cell_at(Mass::new(800), 12000, -22000)
        .expect("spawn dumper");
    world
        .dump_matter(dumper, Mass::new(400))
        .expect("dump crumb");
    world
        .set_program(dumper, compile_text(DEMO_SIT).expect("sit"))
        .expect("program dumper");
}

fn spawn_tick_loop(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(MS_PER_TICK));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let json = {
                let mut world = state.world.lock().await;
                world.tick();
                // Skip serialize work when nobody is watching.
                if state.snapshots.receiver_count() == 0 {
                    continue;
                }
                wrap_snapshot(&world.snapshot_json())
            };
            let _ = state.snapshots.send(json);
        }
    });
}

async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    match read_index_with_env(&state.skin_dir, &state.env) {
        Ok(body) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))],
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

    {
        let snap = wrap_snapshot(&state.world.lock().await.snapshot_json());
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
        ClientMsg::SetProgram { cell_id, source } => {
            let program = match compile_text(&source) {
                Ok(p) => p,
                Err(err) => {
                    return Some(
                        serde_json::json!({
                            "type": "error",
                            "message": err.to_string(),
                        })
                        .to_string(),
                    );
                }
            };
            let mut world = state.world.lock().await;
            match world.set_program(CellId::from_raw(cell_id), program) {
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
                        "message": err.to_string(),
                    })
                    .to_string(),
                ),
            }
        }
        ClientMsg::Reset => {
            let mut world = state.world.lock().await;
            seed_world(&mut world);
            let snap = wrap_snapshot(&world.snapshot_json());
            let _ = state.snapshots.send(snap);
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

fn wrap_snapshot(raw_object: &str) -> String {
    // raw_object is `{...}`; nest under type without re-parse.
    let mut out = String::from("{\"type\":\"snapshot\",\"world\":");
    out.push_str(raw_object);
    out.push('}');
    out
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
