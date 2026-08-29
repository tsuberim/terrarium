//! Dev-only: embed compiled strategy WASM as WAT in kernel + skin examples.

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
        "predator",
        "Predator",
        "Hunts creatures in vision — eats when adjacent",
        "PREDATOR",
    )?;
    sync_one(
        &repo,
        &target.join("strategy_scavenger.wasm"),
        "scavenger",
        "Scavenger",
        "Hunts corpses in vision — eats when adjacent",
        "SCAVENGER",
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
    let code_start = ts[start..]
        .find("code: `")
        .ok_or("missing code")?
        + start
        + 7;
    let code_end = ts[code_start..].find("`,\n").ok_or("missing code end")? + code_start;
    let ts_wat = escape_ts_template(&wat);
    let indented: String = ts_wat
        .lines()
        .map(|l| if l.is_empty() { String::new() } else { format!("  {l}") })
        .collect::<Vec<_>>()
        .join("\n");
    let head = format!(
        "id: \"{id}\",\n    name: \"{name}\",\n    description: \"{description}\",\n    code: `{indented}\n`,"
    );
    ts = format!("{}{}{}", &ts[..start], head, &ts[code_end + 2..]);
    fs::write(&ts_path, ts)?;

    println!("synced {id} ({} wasm bytes)", wasm.len());
    Ok(())
}

fn escape_ts_template(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${")
}
