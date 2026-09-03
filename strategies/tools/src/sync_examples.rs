//! Dev-only: embed compiled strategy WASM in sim examples.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let strategies_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("..");
    let repo = strategies_root.join("..");
    let target = strategies_root.join("target/wasm32-unknown-unknown/release");

    sync_one(
        &repo,
        &target.join("strategy_predator.wasm"),
        "PREDATOR",
    )?;
    sync_one(
        &repo,
        &target.join("strategy_scavenger.wasm"),
        "SCAVENGER",
    )?;
    sync_one(&repo, &target.join("strategy_prey.wasm"), "PREY")?;
    sync_one(&repo, &target.join("strategy_hawk.wasm"), "HAWK")?;

    Ok(())
}

fn sync_one(repo: &Path, wasm_path: &Path, const_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let wasm = fs::read(wasm_path).map_err(|e| {
        format!(
            "missing {}: {e} (run ./scripts/build-strategies.sh first)",
            wasm_path.display()
        )
    })?;
    let wat = wasmprinter::print_bytes(&wasm)?;

    let rust_path = repo.join("crates/sim/src/examples.rs");
    let mut rust = fs::read_to_string(&rust_path)?;
    let rust_start = format!("pub(crate) const {const_name}: &str = r#\"");
    let (pre, rest) = rust
        .split_once(&rust_start)
        .ok_or(format!("missing {rust_start}"))?;
    let end = rest.find("\"#;").ok_or("missing WAT end")? + 3;
    rust = format!("{pre}{rust_start}\n{wat}\n\"#;{}", &rest[end..]);
    fs::write(&rust_path, rust)?;

    println!("synced {const_name} ({} wasm bytes)", wasm.len());
    Ok(())
}
