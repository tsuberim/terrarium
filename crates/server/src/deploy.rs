use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use terrarium_kernel::{compile_wat, host, vm::Creature, WatError, CORPSE_ENERGY};
use uuid::Uuid;

use crate::accounts::{account_credits, ensure_account};
use crate::state::AppState;

const MAX_CREATURE_CODE_LEN: usize = 32_768;
const MAX_WASM_BYTES: usize = 64 * 1024;
const MAX_WASM_B64_LEN: usize = 96 * 1024;

#[derive(Serialize)]
pub struct DeployResponse {
    pub id: String,
    pub x: i64,
    pub y: i64,
    pub energy: i64,
    pub credits: i64,
}

pub enum DeployError {
    InsufficientCredits,
    Occupied,
    InvalidEnergy(String),
    InvalidProgram(String),
    Internal(anyhow::Error),
}

fn deploy_internal(err: impl Into<anyhow::Error>) -> DeployError {
    DeployError::Internal(err.into())
}

pub async fn deploy_creature(
    state: &AppState,
    uid: &str,
    x: i64,
    y: i64,
    code: &str,
    extra: i64,
    wasm_b64: Option<&str>,
) -> Result<DeployResponse, DeployError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(DeployError::InvalidProgram("program is required".into()));
    }

    let wasm = if let Some(b64) = wasm_b64 {
        if b64.len() > MAX_WASM_B64_LEN {
            return Err(DeployError::InvalidProgram("wasm too long".into()));
        }
        if code.len() > 4096 {
            return Err(DeployError::InvalidProgram("program too long".into()));
        }
        let bytes = STANDARD
            .decode(b64.trim())
            .map_err(|_| DeployError::InvalidProgram("invalid wasm encoding".into()))?;
        if bytes.is_empty() || bytes.len() > MAX_WASM_BYTES {
            return Err(DeployError::InvalidProgram("invalid wasm size".into()));
        }
        if host::load_module(host::wasm_engine(), &bytes).is_none() {
            return Err(DeployError::InvalidProgram("invalid wasm module".into()));
        }
        bytes
    } else {
        if code.len() > MAX_CREATURE_CODE_LEN {
            return Err(DeployError::InvalidProgram("program too long".into()));
        }
        compile_wat(code).map_err(|err| {
            DeployError::InvalidProgram(match err {
                WatError::Empty => "program is required".into(),
                WatError::Parse(msg) => msg,
            })
        })?
    };

    let min_extra = state.config.deploy_cost;
    if extra < min_extra {
        return Err(DeployError::InvalidEnergy(format!(
            "extra energy must be at least {min_extra}"
        )));
    }

    let cost = CORPSE_ENERGY + extra;
    let id = Uuid::new_v4().to_string();
    let energy = cost;

    if !state.engine.is_deployable(x, y) {
        return Err(DeployError::Occupied);
    }

    let mut tx = state.db.begin().await.map_err(deploy_internal)?;

    ensure_account(&state.db, uid)
        .await
        .map_err(deploy_internal)?;

    let credits: i64 = sqlx::query_scalar("SELECT credits FROM accounts WHERE firebase_uid = ?")
        .bind(uid)
        .fetch_one(&mut *tx)
        .await
        .map_err(deploy_internal)?;

    if credits < cost {
        return Err(DeployError::InsufficientCredits);
    }

    sqlx::query("UPDATE accounts SET credits = credits - ? WHERE firebase_uid = ?")
        .bind(cost)
        .bind(uid)
        .execute(&mut *tx)
        .await
        .map_err(deploy_internal)?;

    let born_tick = state.engine.current_tick() as i64;
    let sim = state.engine.sim_config();

    sqlx::query(
        "INSERT INTO creatures (id, owner_uid, x, y, energy, health, max_health, code, bytecode, born_tick, pc, stack) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, x'')",
    )
    .bind(&id)
    .bind(uid)
    .bind(x)
    .bind(y)
    .bind(energy)
    .bind(sim.max_health as i64)
    .bind(sim.max_health as i64)
    .bind(code)
    .bind(&wasm)
    .bind(born_tick)
    .execute(&mut *tx)
    .await
    .map_err(|err| {
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.is_unique_violation() {
                return DeployError::Occupied;
            }
        }
        DeployError::Internal(err.into())
    })?;

    tx.commit().await.map_err(deploy_internal)?;

    state
        .engine
        .insert_creature(Creature {
            id: id.clone(),
            owner_uid: uid.to_string(),
            x: x as i32,
            y: y as i32,
            energy,
            health: sim.max_health,
            max_health: sim.max_health,
            parent_id: None,
            wasm,
            code: code.to_string(),
            alive: true,
            inbox: vec![],
            death_reason: None,
            born_tick: born_tick as u64,
            facing: 0,
        })
        .map_err(deploy_internal)?;

    let credits = account_credits(&state.db, uid)
        .await
        .map_err(deploy_internal)?;

    Ok(DeployResponse {
        id,
        x,
        y,
        energy,
        credits,
    })
}
