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
use terrarium_test_spec::parse_tests;

const MAX_SOURCE_LEN: usize = 32_768;
const MAX_WASM_BYTES: usize = 64 * 1024;
const COMPILE_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Deserialize)]
pub struct CompileRequest {
    pub language: String,
    pub source: String,
    #[serde(default)]
    pub tests: String,
}

#[derive(Serialize, Clone)]
pub struct Diagnostic {
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
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

    match compile_rust(&body.source, &body.tests).await {
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
                area: None,
            }],
        }),
    )
}

async fn compile_rust(source: &str, tests: &str) -> Result<CompileResponse, CompileResponse> {
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
                    area: Some("source".into()),
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
                area: Some("source".into()),
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
                area: Some("source".into()),
            }],
        });
    }

    let diagnostics = validate_tests(tests);
    let test_errors = diagnostics.iter().any(|d| d.level == "error");

    Ok(CompileResponse {
        ok: !test_errors,
        wasm_b64: Some(STANDARD.encode(wasm)),
        diagnostics,
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
    fs::write(work.join("src/lib.rs"), CRATE_LIB_RS).map_err(|err| compile_error(err.to_string()))?;
    if let Err(diags) = validate_user_module(source) {
        return Err(CompileResponse {
            ok: false,
            wasm_b64: None,
            diagnostics: diags,
        });
    }
    let (user_module, line_offset) = prepare_user_module(source);
    fs::write(work.join("src/user.rs"), user_module).map_err(|err| compile_error(err.to_string()))?;

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
                area: Some("source".into()),
            });
        }
        remap_user_diagnostics(&mut diagnostics, line_offset);
        return Err(CompileResponse {
            ok: false,
            wasm_b64: None,
            diagnostics,
        });
    }

    Ok(())
}

/// Lines prepended when the editor source omits `use terrarium_sdk::…`.
const PRELUDE_LINE_OFFSET: u32 = 2;

const CRATE_LIB_RS: &str = r#"#![no_std]

mod user;

#[no_mangle]
pub extern "C" fn main() {
    user::main();
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
"#;

fn creature_body(source: &str) -> String {
    source.replace("\r\n", "\n").trim().to_string()
}

fn validate_tests(tests: &str) -> Vec<Diagnostic> {
    if tests.trim().is_empty() {
        return vec![Diagnostic {
            level: "error".into(),
            message: "tests tab is empty — add at least one #[terrarium::test]".into(),
            line: Some(1),
            column: None,
            area: Some("tests".into()),
        }];
    }
    let parsed = parse_tests(tests);
    parsed
        .diagnostics
        .into_iter()
        .map(|d| Diagnostic {
            level: d.level,
            message: d.message,
            line: d.line,
            column: d.column,
            area: d.area.or(Some("tests".into())),
        })
        .collect()
}

fn validate_user_module(source: &str) -> Result<(), Vec<Diagnostic>> {
    let body = creature_body(source);
    if body.is_empty() {
        return Err(vec![Diagnostic {
            level: "error".into(),
            message: "Write creature logic — e.g. move_forward(); or loop { ... }.".into(),
            line: Some(1),
            column: None,
            area: Some("source".into()),
        }]);
    }
    if body.contains("fn tick") {
        return Err(vec![Diagnostic {
            level: "error".into(),
            message: "Use a lifetime program (statements or loop { ... }), not pub fn tick().".into(),
            line: Some(1),
            column: None,
            area: Some("source".into()),
        }]);
    }
    Ok(())
}

fn strip_prelude_import(body: &str) -> String {
    body.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with("use terrarium_sdk")
                || trimmed.starts_with("pub use terrarium_sdk"))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn wrap_as_main(body: &str) -> String {
    if body.contains("fn main") {
        return body.to_string();
    }
    format!("pub fn main() {{\n{body}\n}}")
}

fn prepare_user_module(source: &str) -> (String, u32) {
    let body = strip_prelude_import(&creature_body(source));
    (
        format!(
            "use terrarium_sdk::prelude::*;\n\n{}\n",
            wrap_as_main(&body)
        ),
        PRELUDE_LINE_OFFSET,
    )
}

fn remap_user_diagnostics(diagnostics: &mut [Diagnostic], line_offset: u32) {
    for d in diagnostics {
        d.area = Some("source".into());
        if line_offset == 0 {
            continue;
        }
        if let Some(line) = d.line {
            if line > line_offset {
                d.line = Some(line - line_offset);
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
            area: Some("source".into()),
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
            area: Some("source".into()),
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
    fn creature_body_trims_source() {
        let src = "let x = 1;\n";
        assert_eq!(creature_body(src), "let x = 1;");
    }

    #[test]
    fn prepare_user_module_wraps_loop_as_main() {
        let src = "loop {\n    move_forward();\n    rotate(1);\n}\n";
        let (module, offset) = prepare_user_module(src);
        assert!(module.contains("pub fn main()"));
        assert!(module.contains("loop {"));
        assert!(module.contains("move_forward()"));
        assert!(!module.contains("__terrarium_pc"));
        assert_eq!(offset, PRELUDE_LINE_OFFSET);
        assert!(CRATE_LIB_RS.contains("user::main()"));
    }

    #[test]
    fn validate_user_module_accepts_bare_statements() {
        assert!(validate_user_module("move_forward();").is_ok());
        assert!(validate_user_module("pub fn main() {}").is_ok());
        assert!(validate_user_module("loop { move_forward(); }").is_ok());
        assert!(validate_user_module("   ").is_err());
        assert!(validate_user_module("pub fn tick() {}").is_err());
    }

    #[test]
    fn prepare_user_module_wraps_bare_statement() {
        let (module, offset) = prepare_user_module("move_forward();");
        assert!(module.contains("use terrarium_sdk::prelude::*"));
        assert!(module.contains("pub fn main()"));
        assert!(!module.contains("loop {"));
        assert!(module.contains("move_forward();"));
        assert_eq!(offset, PRELUDE_LINE_OFFSET);
    }

    #[test]
    fn prepare_user_module_strips_user_prelude() {
        let (module, offset) = prepare_user_module("use terrarium_sdk::prelude::*;\n\nmove_forward();");
        assert_eq!(module.matches("use terrarium_sdk::prelude::*;").count(), 1);
        assert!(module.contains("move_forward();"));
        assert_eq!(offset, PRELUDE_LINE_OFFSET);
    }

    #[test]
    fn remap_user_diagnostics_shifts_prelude_only() {
        let mut diags = vec![Diagnostic {
            level: "error".into(),
            message: "x".into(),
            line: Some(4),
            column: Some(1),
            area: None,
        }];
        remap_user_diagnostics(&mut diags, PRELUDE_LINE_OFFSET);
        assert_eq!(diags[0].line, Some(2));
    }
}
