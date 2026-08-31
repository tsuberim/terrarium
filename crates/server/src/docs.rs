use axum::{
    http::header,
    response::{Html, IntoResponse, Response},
};

pub const OPENAPI: &str = include_str!("openapi.json");

fn scalar_html(openapi_url: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Terrarium API</title>
</head>
<body>
  <script id="api-reference" data-url="{openapi_url}"></script>
  <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#
    )
}

pub async fn scalar() -> Html<String> {
    Html(scalar_html("/openapi.json"))
}

pub async fn scalar_api() -> Html<String> {
    Html(scalar_html("/api/openapi.json"))
}

pub async fn openapi_raw() -> Response {
    ([(header::CONTENT_TYPE, "application/json")], OPENAPI).into_response()
}
