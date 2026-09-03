use base64::{engine::general_purpose::STANDARD, Engine as _};
use terrarium_sim::{run_sandbox, SandboxRequest, SandboxResult, SandboxScenario, SimConfig};

const MAX_WASM_B64_LEN: usize = 96 * 1024;

pub fn decode_wasm(b64: &str) -> Result<Vec<u8>, String> {
    if b64.len() > MAX_WASM_B64_LEN {
        return Err("wasm too long".into());
    }
    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|_| "invalid wasm encoding".to_string())?;
    if bytes.is_empty() {
        return Err("empty wasm".into());
    }
    Ok(bytes)
}

pub fn run_creature_sandbox(
    wasm: &[u8],
    scenario: SandboxScenario,
    ticks: u64,
    config: Option<SimConfig>,
) -> SandboxResult {
    run_sandbox(SandboxRequest {
        wasm,
        scenario,
        ticks,
        config,
        start_energy: None,
    })
}
