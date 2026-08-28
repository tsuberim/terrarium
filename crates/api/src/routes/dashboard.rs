use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::auth::bearer_from_header;
use crate::db::ApiTokenRow;
use crate::error::{ApiError, ApiResult};
use crate::scopes::TokenScopes;
use crate::AppState;

async fn session_account(headers: &HeaderMap, state: &AppState) -> ApiResult<String> {
    let token = bearer_from_header(headers.get("authorization").and_then(|v| v.to_str().ok()))
        .ok_or(ApiError::Unauthorized)?;
    state.identity.authenticate_human(token, &state.db).await
}

#[derive(Serialize)]
pub struct ConfigResponse {
    environment: String,
    free_mint_enabled: bool,
    dev_auth_enabled: bool,
    firebase: Option<FirebaseConfigResponse>,
    api_base: String,
}

#[derive(Serialize)]
pub struct FirebaseConfigResponse {
    api_key: String,
    auth_domain: String,
    project_id: String,
}

pub async fn config(State(state): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        environment: state.config.env.name().to_string(),
        free_mint_enabled: state.config.env.allows_free_mint(),
        dev_auth_enabled: state.identity.dev_auth_enabled() && !state.identity.firebase_configured(),
        firebase: state.config.firebase_web.as_ref().map(|f| FirebaseConfigResponse {
            api_key: f.api_key.clone(),
            auth_domain: f.auth_domain.clone(),
            project_id: f.project_id.clone(),
        }),
        api_base: String::new(),
    })
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<MeResponse>> {
    let account_id = session_account(&headers, &state).await?;
    let credits = state.db.credits(&account_id)?;
    Ok(Json(MeResponse {
        account_id,
        credits,
        environment: state.config.env.name().to_string(),
        free_mint_enabled: state.config.env.allows_free_mint(),
        billing_enabled: false,
        auth_mode: if state.identity.firebase_configured() {
            "firebase"
        } else {
            "dev"
        }
        .to_string(),
    }))
}

#[derive(Serialize)]
pub struct MeResponse {
    account_id: String,
    credits: u64,
    environment: String,
    free_mint_enabled: bool,
    billing_enabled: bool,
    auth_mode: String,
}

pub async fn list_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ApiTokenRow>>> {
    let account_id = session_account(&headers, &state).await?;
    Ok(Json(state.db.list_api_tokens(&account_id)?))
}

#[derive(Deserialize)]
pub struct MintTokenRequest {
    #[serde(default = "default_label")]
    pub label: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

fn default_label() -> String {
    "default".to_string()
}

#[derive(Serialize)]
pub struct MintTokenResponse {
    id: String,
    token: String,
    label: String,
    scopes: String,
}

pub async fn mint_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MintTokenRequest>,
) -> ApiResult<Json<MintTokenResponse>> {
    let account_id = session_account(&headers, &state).await?;
    let scopes = TokenScopes::from_request(&body.scopes)?;
    let (id, token) = state.db.mint_api_token(&account_id, &body.label, scopes)?;
    Ok(Json(MintTokenResponse {
        id,
        token,
        label: body.label,
        scopes: scopes.to_string(),
    }))
}

pub async fn revoke_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let account_id = session_account(&headers, &state).await?;
    state.db.revoke_api_token(&account_id, &id)?;
    Ok(Json(serde_json::json!({ "revoked": id })))
}

#[derive(Deserialize)]
pub struct FaucetRequest {
    #[serde(default)]
    pub amount: Option<u64>,
}

#[derive(Serialize)]
pub struct FaucetResponse {
    credits: u64,
    minted: u64,
}

pub async fn faucet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<FaucetRequest>,
) -> ApiResult<Json<FaucetResponse>> {
    if !state.config.env.allows_free_mint() {
        return Err(ApiError::Forbidden);
    }
    let account_id = session_account(&headers, &state).await?;
    let amount = body.amount.unwrap_or(state.config.faucet_amount);
    if amount == 0 {
        return Err(ApiError::BadRequest("amount must be greater than zero".into()));
    }
    let credits = state
        .db
        .faucet_credits(&account_id, amount, "staging_faucet")?;
    Ok(Json(FaucetResponse {
        credits,
        minted: amount,
    }))
}

#[derive(Serialize)]
pub struct BillingCheckoutResponse {
    status: &'static str,
    message: String,
}

pub async fn billing_checkout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<BillingCheckoutResponse>> {
    let _account = session_account(&headers, &state).await?;
    Ok(Json(BillingCheckoutResponse {
        status: "coming_soon",
        message: format!(
            "Stripe checkout is not wired yet (env: {}). Use the free faucet on staging/local.",
            state.config.env.name()
        ),
    }))
}
