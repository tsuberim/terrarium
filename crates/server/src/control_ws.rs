//! Control WebSocket — API-key attach to owned creatures.

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::error::RecvError;

use crate::accounts::account_creature_id;
use crate::auth::AuthenticatedUser;
use crate::control::{AttachError, ControlClientMsg, ControlRegistry, ControlServerMsg};
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct ControlQuery {
    #[serde(deserialize_with = "crate::wire::deserialize_u64")]
    creature_id: u64,
}

pub async fn control_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<ControlQuery>,
) -> Response {
    let creature_id = query.creature_id;
    match state.engine.creature_owned_by(creature_id, &user.uid) {
        Some(info) if info.alive => {}
        Some(_) => {
            return (
                StatusCode::GONE,
                axum::Json(serde_json::json!({ "error": "creature not alive" })),
            )
                .into_response();
        }
        None => {
            return (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({ "error": "not your creature" })),
            )
                .into_response();
        }
    }

    let account_creature_id = match account_creature_id(&state.db, &user.uid).await {
        Ok(id) => id,
        Err(err) => {
            tracing::error!(error = %err, "account_creature_id failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let sub = match state.control.try_attach(creature_id) {
        Ok(rx) => rx,
        Err(AttachError::TooManySessions) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({ "error": AttachError::TooManySessions.code() })),
            )
                .into_response();
        }
    };

    ws.on_upgrade(move |socket| {
        handle_control_socket(
            socket,
            state.engine.clone(),
            state.control.clone(),
            creature_id,
            account_creature_id,
            sub,
        )
    })
}

async fn handle_control_socket(
    socket: WebSocket,
    engine: Arc<crate::engine::WorldEngine>,
    control: Arc<ControlRegistry>,
    creature_id: u64,
    account_creature_id: u64,
    mut sub: tokio::sync::broadcast::Receiver<ControlServerMsg>,
) {
    let (mut sender, mut receiver) = socket.split();

    let attached = ControlServerMsg::Attached {
        creature_id,
        account_creature_id,
        tick: engine.current_tick(),
    };
    if send_json(&mut sender, &attached).await.is_err() {
        control.detach(creature_id);
        return;
    }

    let mut msgs_this_tick = 0u32;
    let mut last_tick = engine.current_tick();

    loop {
        tokio::select! {
            msg = sub.recv() => {
                match msg {
                    Ok(server_msg) => {
                        if send_json(&mut sender, &server_msg).await.is_err() {
                            break;
                        }
                        if matches!(server_msg, ControlServerMsg::Detached { .. }) {
                            break;
                        }
                    }
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => break,
                }
            }
            incoming = receiver.next() => {
                let Some(incoming) = incoming else { break };
                let Ok(raw) = incoming else { break };
                match raw {
                    Message::Close(_) => break,
                    Message::Ping(p) => {
                        if sender.send(Message::Pong(p)).await.is_err() {
                            break;
                        }
                    }
                    Message::Text(text) => {
                        if text.len() > 16_384 {
                            let _ = send_json(&mut sender, &ControlServerMsg::Error {
                                code: "payload_too_large".into(),
                                message: "message too large".into(),
                            }).await;
                            break;
                        }
                        let parsed: ControlClientMsg = match serde_json::from_str(&text) {
                            Ok(v) => v,
                            Err(err) => {
                                let _ = send_json(&mut sender, &ControlServerMsg::Error {
                                    code: "invalid_json".into(),
                                    message: err.to_string(),
                                }).await;
                                continue;
                            }
                        };
                        if matches!(parsed, ControlClientMsg::Ping) {
                            let _ = send_json(&mut sender, &ControlServerMsg::Pong).await;
                            continue;
                        }
                        let tick = engine.current_tick();
                        if tick != last_tick {
                            last_tick = tick;
                            msgs_this_tick = 0;
                        }
                        if msgs_this_tick >= 32 {
                            let _ = send_json(&mut sender, &ControlServerMsg::Error {
                                code: "rate_limited".into(),
                                message: "max 32 control messages per tick".into(),
                            }).await;
                            continue;
                        }
                        match handle_client_msg(&engine, creature_id, parsed).await {
                            Ok(()) => msgs_this_tick += 1,
                            Err((code, message)) => {
                                let _ = send_json(&mut sender, &ControlServerMsg::Error { code, message }).await;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    control.detach(creature_id);
}

async fn handle_client_msg(
    engine: &crate::engine::WorldEngine,
    creature_id: u64,
    msg: ControlClientMsg,
) -> Result<(), (String, String)> {
    match msg {
        ControlClientMsg::Ping => Ok(()),
        ControlClientMsg::Signal { target, envelope } => {
            let payload = envelope
                .to_payload()
                .map_err(|code| (code.into(), code.into()))?;
            engine
                .inject_control_signal(creature_id, target, payload)
                .map_err(|code| (code.into(), code.into()))
        }
        ControlClientMsg::Broadcast { envelope } => {
            let payload = envelope
                .to_payload()
                .map_err(|code| (code.into(), code.into()))?;
            engine
                .inject_control_broadcast(creature_id, payload)
                .map_err(|code| (code.into(), code.into()))
        }
    }
}

async fn send_json(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    msg: &ControlServerMsg,
) -> Result<(), ()> {
    let text = serde_json::to_string(msg).map_err(|_| ())?;
    sender
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}
