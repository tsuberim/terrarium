use std::sync::Arc;

use axum::body::Body;
use http_body_util::BodyExt;
use terrarium_api::{app, AppState, Config, Db, WorldHost};
use tower::ServiceExt;

fn test_state(env: &str) -> AppState {
    let mut config = Config::from_env();
    config.env = terrarium_api::Environment::parse(env);
    config.database_path = std::path::PathBuf::from(":memory:");
    config.dashboard_dir = std::path::PathBuf::from("apps/dashboard");
    AppState {
        db: Arc::new(Db::open(&config.database_path).unwrap()),
        world: Arc::new(WorldHost::new()),
        config,
    }
}

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn spawn_flow_staging_faucet() {
    let state = test_state("staging");
    let app = app(state);

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/v1/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let account = body_json(res.into_body()).await;
    let session = account["session_token"].as_str().unwrap();

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/dashboard/api/faucet")
                .header("Authorization", format!("Bearer {session}"))
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/dashboard/api/tokens")
                .header("Authorization", format!("Bearer {session}"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"label":"ci"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let token_body = body_json(res.into_body()).await;
    let api_token = token_body["token"].as_str().unwrap();

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/v1/spawn")
                .header("Authorization", format!("Bearer {api_token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"mass":100,"x":0,"y":0}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let spawn = body_json(res.into_body()).await;
    assert_eq!(spawn["mass"], 100);
    assert!(spawn["cell_id"].is_number());
}

#[tokio::test]
async fn production_faucet_forbidden() {
    let state = test_state("production");
    let app = app(state);

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/v1/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let account = body_json(res.into_body()).await;
    let session = account["session_token"].as_str().unwrap();

    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/dashboard/api/faucet")
                .header("Authorization", format!("Bearer {session}"))
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn spawn_insufficient_credits() {
    let state = test_state("staging");
    let app = app(state);

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/v1/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let account = body_json(res.into_body()).await;
    let session = account["session_token"].as_str().unwrap();

    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/dashboard/api/tokens")
                .header("Authorization", format!("Bearer {session}"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"label":"x"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let token_body = body_json(res.into_body()).await;
    let api_token = token_body["token"].as_str().unwrap();

    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/v1/spawn")
                .header("Authorization", format!("Bearer {api_token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"mass":100}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 402);
}

#[tokio::test]
async fn bad_token_unauthorized() {
    let state = test_state("local");
    let app = app(state);

    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/v1/spawn")
                .header("Authorization", "Bearer trm_bad")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"mass":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
