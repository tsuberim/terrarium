use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::auth::AuthenticatedUser;

use crate::api_keys;
use crate::auth;
use crate::state::AppState;

pub async fn dev_only(req: Request, next: Next) -> Response {
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

pub async fn require_user(
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

pub async fn require_firebase_user(
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
