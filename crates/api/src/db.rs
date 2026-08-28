use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};
use terrarium_kernel::{compile_text, Mass, World};
use uuid::Uuid;

use crate::auth::{generate_token, hash_token};
use crate::error::{ApiError, ApiResult};
use crate::scopes::TokenScopes;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &std::path::Path) -> ApiResult<Self> {
        let conn = Connection::open(path).map_err(|_| ApiError::Internal)?;
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY NOT NULL,
                credits INTEGER NOT NULL DEFAULT 0 CHECK(credits >= 0),
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS session_tokens (
                token_hash TEXT PRIMARY KEY NOT NULL,
                account_id TEXT NOT NULL REFERENCES accounts(id),
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS api_tokens (
                id TEXT PRIMARY KEY NOT NULL,
                account_id TEXT NOT NULL REFERENCES accounts(id),
                token_hash TEXT NOT NULL UNIQUE,
                label TEXT NOT NULL,
                scopes TEXT NOT NULL DEFAULT 'spawn,read',
                created_at TEXT NOT NULL,
                revoked_at TEXT
            );

            CREATE TABLE IF NOT EXISTS credit_ledger (
                id TEXT PRIMARY KEY NOT NULL,
                account_id TEXT NOT NULL REFERENCES accounts(id),
                delta INTEGER NOT NULL,
                reason TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS spawns (
                id TEXT PRIMARY KEY NOT NULL,
                account_id TEXT NOT NULL REFERENCES accounts(id),
                cell_id INTEGER NOT NULL,
                mass INTEGER NOT NULL,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
            ",
        )
        .map_err(|_| ApiError::Internal)?;
        let _ = conn.execute(
            "ALTER TABLE api_tokens ADD COLUMN scopes TEXT NOT NULL DEFAULT 'spawn,read'",
            [],
        );
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn now_iso() -> String {
        // Stable enough for v1; no chrono dependency.
        format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
    }

    pub fn ensure_account(&self, account_id: &str) -> ApiResult<()> {
        let now = Self::now_iso();
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        conn.execute(
            "INSERT OR IGNORE INTO accounts (id, credits, created_at) VALUES (?1, 0, ?2)",
            params![account_id, now],
        )
        .map_err(|_| ApiError::Internal)?;
        Ok(())
    }

    pub fn create_dev_account(&self) -> ApiResult<(String, String)> {
        let account_id = Uuid::new_v4().to_string();
        let session_raw = generate_token("trm_sess_");
        let session_hash = hash_token(&session_raw);
        let now = Self::now_iso();
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        conn.execute(
            "INSERT INTO accounts (id, credits, created_at) VALUES (?1, 0, ?2)",
            params![account_id, now],
        )
        .map_err(|_| ApiError::Internal)?;
        conn.execute(
            "INSERT INTO session_tokens (token_hash, account_id, created_at) VALUES (?1, ?2, ?3)",
            params![session_hash, account_id, now],
        )
        .map_err(|_| ApiError::Internal)?;
        Ok((account_id, session_raw))
    }

    pub fn account_for_session(&self, session_raw: &str) -> ApiResult<String> {
        let hash = hash_token(session_raw);
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        conn.query_row(
            "SELECT account_id FROM session_tokens WHERE token_hash = ?1",
            params![hash],
            |row| row.get(0),
        )
        .map_err(|_| ApiError::Unauthorized)
    }

    pub fn account_for_api_token(&self, token_raw: &str) -> ApiResult<(String, TokenScopes)> {
        let hash = hash_token(token_raw);
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        conn.query_row(
            "SELECT account_id, scopes FROM api_tokens WHERE token_hash = ?1 AND revoked_at IS NULL",
            params![hash],
            |row| {
                let account_id: String = row.get(0)?;
                let scopes_raw: String = row.get(1)?;
                Ok((account_id, TokenScopes::from_db(&scopes_raw)))
            },
        )
        .map_err(|_| ApiError::Unauthorized)
    }

    pub fn credits(&self, account_id: &str) -> ApiResult<u64> {
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        let credits: i64 = conn
            .query_row(
                "SELECT credits FROM accounts WHERE id = ?1",
                params![account_id],
                |row| row.get(0),
            )
            .map_err(|_| ApiError::NotFound)?;
        Ok(credits.max(0) as u64)
    }

    pub fn mint_api_token(
        &self,
        account_id: &str,
        label: &str,
        scopes: TokenScopes,
    ) -> ApiResult<(String, String)> {
        let id = Uuid::new_v4().to_string();
        let raw = generate_token("trm_");
        let hash = hash_token(&raw);
        let now = Self::now_iso();
        let scopes_db = scopes.to_db_string();
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        conn.execute(
            "INSERT INTO api_tokens (id, account_id, token_hash, label, scopes, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, account_id, hash, label, scopes_db, now],
        )
        .map_err(|_| ApiError::Internal)?;
        Ok((id, raw))
    }

    pub fn list_api_tokens(&self, account_id: &str) -> ApiResult<Vec<ApiTokenRow>> {
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, label, scopes, created_at, revoked_at FROM api_tokens WHERE account_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|_| ApiError::Internal)?;
        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok(ApiTokenRow {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    scopes: row.get(2)?,
                    created_at: row.get(3)?,
                    revoked_at: row.get(4)?,
                })
            })
            .map_err(|_| ApiError::Internal)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|_| ApiError::Internal)
    }

    pub fn revoke_api_token(&self, account_id: &str, token_id: &str) -> ApiResult<()> {
        let now = Self::now_iso();
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        let updated = conn
            .execute(
                "UPDATE api_tokens SET revoked_at = ?1 WHERE id = ?2 AND account_id = ?3 AND revoked_at IS NULL",
                params![now, token_id, account_id],
            )
            .map_err(|_| ApiError::Internal)?;
        if updated == 0 {
            return Err(ApiError::NotFound);
        }
        Ok(())
    }

    pub fn faucet_credits(&self, account_id: &str, amount: u64, reason: &str) -> ApiResult<u64> {
        self.adjust_credits(account_id, amount as i64, reason)
    }

    fn adjust_credits(&self, account_id: &str, delta: i64, reason: &str) -> ApiResult<u64> {
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        let tx = conn.unchecked_transaction().map_err(|_| ApiError::Internal)?;
        let current: i64 = tx
            .query_row(
                "SELECT credits FROM accounts WHERE id = ?1",
                params![account_id],
                |row| row.get(0),
            )
            .map_err(|_| ApiError::NotFound)?;
        let next = current
            .checked_add(delta)
            .ok_or(ApiError::BadRequest("credit overflow".into()))?;
        if next < 0 {
            return Err(ApiError::InsufficientCredits);
        }
        tx.execute(
            "UPDATE accounts SET credits = ?1 WHERE id = ?2",
            params![next, account_id],
        )
        .map_err(|_| ApiError::Internal)?;
        let entry_id = Uuid::new_v4().to_string();
        let now = Self::now_iso();
        tx.execute(
            "INSERT INTO credit_ledger (id, account_id, delta, reason, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entry_id, account_id, delta, reason, now],
        )
        .map_err(|_| ApiError::Internal)?;
        tx.commit().map_err(|_| ApiError::Internal)?;
        Ok(next as u64)
    }

    pub fn spend_and_record_spawn(
        &self,
        account_id: &str,
        mass: u64,
        cell_id: u64,
        x: i32,
        y: i32,
    ) -> ApiResult<(String, u64)> {
        let conn = self.conn.lock().map_err(|_| ApiError::Internal)?;
        let tx = conn.unchecked_transaction().map_err(|_| ApiError::Internal)?;
        let current: i64 = tx
            .query_row(
                "SELECT credits FROM accounts WHERE id = ?1",
                params![account_id],
                |row| row.get(0),
            )
            .map_err(|_| ApiError::NotFound)?;
        let cost = mass as i64;
        if current < cost {
            return Err(ApiError::InsufficientCredits);
        }
        let next = current - cost;
        tx.execute(
            "UPDATE accounts SET credits = ?1 WHERE id = ?2",
            params![next, account_id],
        )
        .map_err(|_| ApiError::Internal)?;
        let ledger_id = Uuid::new_v4().to_string();
        let now = Self::now_iso();
        tx.execute(
            "INSERT INTO credit_ledger (id, account_id, delta, reason, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![ledger_id, account_id, -cost, "spawn", now],
        )
        .map_err(|_| ApiError::Internal)?;
        let spawn_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO spawns (id, account_id, cell_id, mass, x, y, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![spawn_id, account_id, cell_id as i64, mass as i64, x, y, now],
        )
        .map_err(|_| ApiError::Internal)?;
        tx.commit().map_err(|_| ApiError::Internal)?;
        Ok((spawn_id, next as u64))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiTokenRow {
    pub id: String,
    pub label: String,
    pub scopes: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
}

pub struct WorldHost {
    inner: Mutex<World>,
}

impl WorldHost {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(World::new()),
        }
    }

    pub fn spawn_cell(
        &self,
        mass: u64,
        x: i32,
        y: i32,
        program: Option<&str>,
    ) -> ApiResult<u64> {
        let mut world = self.inner.lock().map_err(|_| ApiError::Internal)?;
        let cell_id = world
            .spawn_cell_at(Mass::new(mass), x, y)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        if let Some(src) = program {
            if !src.trim().is_empty() {
                let prog = compile_text(src).map_err(|e| ApiError::BadRequest(e.to_string()))?;
                world
                    .set_program(cell_id, prog)
                    .map_err(|e| ApiError::BadRequest(e.to_string()))?;
            }
        }
        Ok(cell_id.get())
    }

    pub fn tick(&self) {
        if let Ok(mut world) = self.inner.lock() {
            world.tick();
        }
    }

    pub fn snapshot_json(&self) -> ApiResult<String> {
        let world = self.inner.lock().map_err(|_| ApiError::Internal)?;
        let s = world.snapshot();
        Ok(serde_json::json!({
            "tick": s.tick,
            "total_mass": s.total_mass.get(),
            "house_burned": s.house_burned.get(),
            "spawned_mass": s.spawned_mass.get(),
            "width": s.width,
            "height": s.height,
            "cells": s.cells.iter().map(|c| serde_json::json!({
                "id": c.id.get(),
                "mass": c.mass.get(),
                "x": c.x,
                "y": c.y,
            })).collect::<Vec<_>>(),
        })
        .to_string())
    }
}

pub fn spawn_tick_loop(host: Arc<WorldHost>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
        loop {
            interval.tick().await;
            host.tick();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_and_spawn_spend() {
        let db = Db::open(std::path::Path::new(":memory:")).unwrap();
        let (account_id, _sess) = db.create_dev_account().unwrap();
        db.faucet_credits(&account_id, 500, "test_faucet").unwrap();
        assert_eq!(db.credits(&account_id).unwrap(), 500);
        let (spawn_id, remaining) = db
            .spend_and_record_spawn(&account_id, 100, 1, 0, 0)
            .unwrap();
        assert!(!spawn_id.is_empty());
        assert_eq!(remaining, 400);
        assert_eq!(db.credits(&account_id).unwrap(), 400);
    }

    #[test]
    fn insufficient_credits_rejected() {
        let db = Db::open(std::path::Path::new(":memory:")).unwrap();
        let (account_id, _sess) = db.create_dev_account().unwrap();
        let err = db
            .spend_and_record_spawn(&account_id, 1, 1, 0, 0)
            .unwrap_err();
        assert!(matches!(err, ApiError::InsufficientCredits));
    }

    #[test]
    fn api_token_auth() {
        let db = Db::open(std::path::Path::new(":memory:")).unwrap();
        let (account_id, _sess) = db.create_dev_account().unwrap();
        let (_id, raw) = db.mint_api_token(&account_id, "test", TokenScopes::all()).unwrap();
        let (id, scopes) = db.account_for_api_token(&raw).unwrap();
        assert_eq!(id, account_id);
        assert!(scopes.spawn && scopes.read);
        assert!(db.account_for_api_token("trm_bad").is_err());
    }
}
