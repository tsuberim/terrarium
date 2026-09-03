use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use axum::http::StatusCode;
use axum::Json;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::timeout;
use uuid::Uuid;

use crate::validate::validate_wasm;

const MAX_SOURCE_LEN: usize = 32_768;
const MAX_WASM_BYTES: usize = 64 * 1024;
const COMPILE_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Deserialize)]
pub struct CompileRequest {
    pub language: String,
    pub source: String,
}

#[derive(Serialize, Clone)]
pub struct Diagnostic {
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

#[derive(Serialize)]
pub struct CompileResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wasm_b64: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

pub async fn compile(
    Json(body): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, (StatusCode, Json<CompileResponse>)> {
    if body.language != "rust" {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "only language=rust is supported",
        ));
    }
    if body.source.trim().is_empty() {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "source is required",
        ));
    }
    if body.source.len() > MAX_SOURCE_LEN {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "source too long",
        ));
    }

    match compile_rust(&body.source).await {
        Ok(res) => Ok(Json(res)),
        Err(res) => Err((StatusCode::BAD_REQUEST, Json(res))),
    }
}

fn err_response(status: StatusCode, message: &str) -> (StatusCode, Json<CompileResponse>) {
    (
        status,
        Json(CompileResponse {
            ok: false,
            wasm_b64: None,
            diagnostics: vec![Diagnostic {
                level: "error".into(),
                message: message.into(),
                line: None,
                column: None,
            }],
        }),
    )
}

async fn compile_rust(source: &str) -> Result<CompileResponse, CompileResponse> {
    let template_dir = template_root();
    let sdk_path = sdk_root();
    let job_id = Uuid::new_v4();
    let work = std::env::temp_dir().join(format!("terrarium-compile-{job_id}"));

    if let Err(err) = run_compile(source, &template_dir, &sdk_path, &work).await {
        let _ = fs::remove_dir_all(&work);
        return Err(err);
    }

    let wasm_path = work.join("target/wasm32-unknown-unknown/release/creature.wasm");
    let wasm = match fs::read(&wasm_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            let _ = fs::remove_dir_all(&work);
            return Err(CompileResponse {
                ok: false,
                wasm_b64: None,
                diagnostics: vec![Diagnostic {
                    level: "error".into(),
                    message: "build succeeded but WASM artifact missing".into(),
                    line: None,
                    column: None,
                }],
            });
        }
    };
    let _ = fs::remove_dir_all(&work);

    if wasm.len() > MAX_WASM_BYTES {
        return Err(CompileResponse {
            ok: false,
            wasm_b64: None,
            diagnostics: vec![Diagnostic {
                level: "error".into(),
                message: format!("WASM too large (max {} bytes)", MAX_WASM_BYTES),
                line: None,
                column: None,
            }],
        });
    }

    if let Err(msg) = validate_wasm(&wasm) {
        return Err(CompileResponse {
            ok: false,
            wasm_b64: None,
            diagnostics: vec![Diagnostic {
                level: "error".into(),
                message: msg,
                line: None,
                column: None,
            }],
        });
    }

    Ok(CompileResponse {
        ok: true,
        wasm_b64: Some(STANDARD.encode(wasm)),
        diagnostics: vec![],
    })
}

async fn run_compile(
    source: &str,
    template_dir: &Path,
    sdk_path: &Path,
    work: &Path,
) -> Result<(), CompileResponse> {
    copy_template(template_dir, work).map_err(|err| compile_error(err.to_string()))?;
    patch_manifest(work, sdk_path).map_err(|err| compile_error(err.to_string()))?;
    fs::write(
        work.join("src/user.rs"),
        wrap_user_source(&creature_body(source)),
    )
    .map_err(|err| compile_error(err.to_string()))?;

    let mut cmd = Command::new("cargo");
    cmd.current_dir(work)
        .env("CARGO_TARGET_DIR", work.join("target"))
        .env("RUSTFLAGS", "-C strip=symbols")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--message-format=json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = timeout(COMPILE_TIMEOUT, cmd.output())
        .await
        .map_err(|_| compile_error("compile timed out".into()))?
        .map_err(|err| compile_error(format!("failed to run cargo: {err}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut diagnostics = parse_cargo_diagnostics(&stdout);

    if !output.status.success() {
        if diagnostics.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            diagnostics.push(Diagnostic {
                level: "error".into(),
                message: stderr.trim().to_string(),
                line: None,
                column: None,
            });
        }
        remap_user_diagnostics(&mut diagnostics);
        return Err(CompileResponse {
            ok: false,
            wasm_b64: None,
            diagnostics,
        });
    }

    Ok(())
}

/// Editor body starts inside `loop {` — offset past prelude + main + loop lines.
const USER_BODY_LINE_OFFSET: u32 = 4;

fn creature_body(source: &str) -> String {
    let normalized = source.replace("\r\n", "\n");
    normalized
        .split("\n---\n")
        .next()
        .or_else(|| normalized.split("\n---").next())
        .unwrap_or(normalized.as_str())
        .trim()
        .to_string()
}

fn wrap_user_source(source: &str) -> String {
    format!(
        "use terrarium_sdk::prelude::*;\n\npub fn main() {{\n    loop {{\n{}\n    }}\n}}\n",
        source.trim_end()
    )
}

fn remap_user_diagnostics(diagnostics: &mut [Diagnostic]) {
    for d in diagnostics {
        if let Some(line) = d.line {
            if line > USER_BODY_LINE_OFFSET {
                d.line = Some(line - USER_BODY_LINE_OFFSET);
            } else {
                d.line = None;
            }
        }
    }
}

fn compile_error(message: String) -> CompileResponse {
    CompileResponse {
        ok: false,
        wasm_b64: None,
        diagnostics: vec![Diagnostic {
            level: "error".into(),
            message,
            line: None,
            column: None,
        }],
    }
}

fn template_root() -> PathBuf {
    std::env::var("TEMPLATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("template"))
}

fn sdk_root() -> PathBuf {
    std::env::var("SDK_PATH").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sdk/rust/terrarium-sdk")
    })
}

fn copy_template(from: &Path, to: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(to.join("src"))?;
    fs::copy(from.join("Cargo.toml"), to.join("Cargo.toml"))?;
    fs::copy(from.join("src/lib.rs"), to.join("src/lib.rs"))?;
    Ok(())
}

fn patch_manifest(work: &Path, sdk_path: &Path) -> anyhow::Result<()> {
    let manifest = fs::read_to_string(work.join("Cargo.toml"))?;
    let sdk = sdk_path.to_string_lossy().replace('\\', "/");
    let patched = manifest.replace("/opt/terrarium-sdk", &sdk);
    fs::write(work.join("Cargo.toml"), patched)?;
    Ok(())
}

fn parse_cargo_diagnostics(stdout: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(|v| v.as_str()) != Some("compiler-message") {
            continue;
        }
        let Some(msg) = value.get("message") else {
            continue;
        };
        let level = msg
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("error")
            .to_string();
        let message = msg
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("compile error")
            .to_string();
        let mut line_no = None;
        let mut col = None;
        if let Some(span) = msg.get("spans").and_then(|s| s.as_array()).and_then(|a| a.first()) {
            line_no = span.get("line_start").and_then(|v| v.as_u64()).map(|n| n as u32);
            col = span
                .get("column_start")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32);
        }
        out.push(Diagnostic {
            level,
            message,
            line: line_no,
            column: col,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rustc_json_line() {
        let sample = r#"{"reason":"compiler-message","message":{"level":"error","message":"expected identifier","spans":[{"line_start":3,"column_start":5}]}}"#;
        let diags = parse_cargo_diagnostics(sample);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, Some(3));
    }

    #[test]
    fn creature_body_splits_on_delimiter() {
        let src = "let x = 1;\n---\n#[terrarium::scenario]\nfn s() {}";
        assert_eq!(creature_body(src), "let x = 1;");
    }

    #[test]
    fn wrap_user_source_injects_prelude_and_main() {
        let wrapped = wrap_user_source("let _ = move_forward();");
        assert!(wrapped.contains("use terrarium_sdk::prelude::*;"));
        assert!(wrapped.contains("pub fn main()"));
        assert!(wrapped.contains("let _ = move_forward();"));
    }

    #[test]
    fn remap_user_diagnostics_shifts_line_numbers() {
        let mut diags = vec![Diagnostic {
            level: "error".into(),
            message: "x".into(),
            line: Some(5),
            column: Some(1),
        }];
        remap_user_diagnostics(&mut diags);
        assert_eq!(diags[0].line, Some(1));
    }
}
