use axum::{
    http::header,
    response::{Html, IntoResponse, Response},
    Json,
};

pub const OPENAPI: &str = include_str!("openapi.json");

pub async fn openapi() -> Json<serde_json::Value> {
    Json(serde_json::from_str(OPENAPI).expect("valid openapi.json"))
}

pub async fn scalar() -> Html<&'static str> {
    Html(SCALAR_HTML)
}

const SCALAR_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Terrarium API</title>
</head>
<body>
  <script id="api-reference" data-url="/openapi.json"></script>
  <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#;

pub async fn scalar_api() -> Html<&'static str> {
    Html(SCALAR_HTML_API)
}

const SCALAR_HTML_API: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Terrarium API</title>
</head>
<body>
  <script id="api-reference" data-url="/api/openapi.json"></script>
  <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#;

pub async fn openapi_raw() -> Response {
    (
        [(header::CONTENT_TYPE, "application/json")],
        OPENAPI,
    )
        .into_response()
}
