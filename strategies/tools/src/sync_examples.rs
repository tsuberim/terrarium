//! Dev-only: embed compiled strategy WASM in kernel + skin examples.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let strategies_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("..");
    let repo = strategies_root.join("..");
    let target = strategies_root.join("target/wasm32-unknown-unknown/release");

    sync_one(
        &repo,
        &target.join("strategy_predator.wasm"),
        "predator",
        "Predator",
        "Hunts prey; eats corpses and food; hunt ping (0x02); clones when energy > 5M",
        "PREDATOR",
    )?;
    sync_one(
        &repo,
        &target.join("strategy_scavenger.wasm"),
        "scavenger",
        "Scavenger",
        "Rushes prey alarms (0x01), forages corpses and food; clones when energy > 5M",
        "SCAVENGER",
    )?;
    sync_one(
        &repo,
        &target.join("strategy_prey.wasm"),
        "prey",
        "Prey",
        "Flees predators, grazes food, alarms (0x01); clones when energy > 5M",
        "PREY",
    )?;
    sync_one(
        &repo,
        &target.join("strategy_hawk.wasm"),
        "hawk",
        "Hawk",
        "Rushes prey alarms (0x01), forages corpses and food; clones when energy > 5M",
        "HAWK",
    )?;

    Ok(())
}

fn sync_one(
    repo: &Path,
    wasm_path: &Path,
    id: &str,
    name: &str,
    description: &str,
    const_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let wasm = fs::read(wasm_path).map_err(|e| {
        format!(
            "missing {}: {e} (run ./scripts/build-strategies.sh first)",
            wasm_path.display()
        )
    })?;
    let wat = wasmprinter::print_bytes(&wasm)?;
    let wasm_b64 = STANDARD.encode(&wasm);

    let rust_path = repo.join("crates/kernel/src/examples.rs");
    let mut rust = fs::read_to_string(&rust_path)?;
    let rust_start = format!("pub(crate) const {const_name}: &str = r#\"");
    let (pre, rest) = rust
        .split_once(&rust_start)
        .ok_or(format!("missing {rust_start}"))?;
    let end = rest.find("\"#;").ok_or("missing WAT end")? + 3;
    rust = format!("{pre}{rust_start}\n{wat}\n\"#;{}", &rest[end..]);
    fs::write(&rust_path, rust)?;

    let ts_path = repo.join("apps/skin/src/lib/examples.ts");
    let mut ts = fs::read_to_string(&ts_path)?;
    let marker = format!("id: \"{id}\",");
    let start = ts.find(&marker).ok_or("missing ts id")?;
    const ENTRY_END: &str = "\n  },";
    let close = ts[start..]
        .find(ENTRY_END)
        .ok_or("missing ts entry end")?
        + start;
    let head = format!(
        "id: \"{id}\",\n    name: \"{name}\",\n    description: \"{description}\",\n    code: \"// {name} (precompiled WASM)\",\n    wasmB64: \"{wasm_b64}\","
    );
    ts = format!("{}{}{}", &ts[..start], head, &ts[close..]);
    fs::write(&ts_path, ts)?;

    println!("synced {id} ({} wasm bytes)", wasm.len());
    Ok(())
}
