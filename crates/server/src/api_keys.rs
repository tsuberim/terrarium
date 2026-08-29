use base64::Engine;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

const MAX_KEYS_PER_USER: i64 = 10;
const KEY_PREFIX: &str = "tr_";

pub fn hash_key(key: &str) -> String {
    let digest = Sha256::digest(key.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn generate_key() -> String {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("rng");
    format!(
        "{KEY_PREFIX}{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    )
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ApiKeyPublic {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct MintApiKeyResponse {
    #[serde(flatten)]
    pub key: ApiKeyPublic,
    /// Full secret — shown once at creation.
    pub secret: String,
}

pub async fn list_keys(db: &SqlitePool, uid: &str) -> anyhow::Result<Vec<ApiKeyPublic>> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT id, name, prefix, created_at, last_used_at FROM api_keys WHERE owner_uid = ? ORDER BY created_at DESC",
    )
    .bind(uid)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, name, prefix, created_at, last_used_at)| ApiKeyPublic {
            id,
            name,
            prefix,
            created_at,
            last_used_at,
        })
        .collect())
}

pub async fn mint_key(
    db: &SqlitePool,
    uid: &str,
    name: &str,
) -> anyhow::Result<MintApiKeyResponse> {
    sqlx::query(
        "INSERT INTO accounts (firebase_uid, credits) VALUES (?, 0) ON CONFLICT DO NOTHING",
    )
    .bind(uid)
    .execute(db)
    .await?;

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE owner_uid = ?")
            .bind(uid)
            .fetch_one(db)
            .await?;
    if count >= MAX_KEYS_PER_USER {
        anyhow::bail!("max {MAX_KEYS_PER_USER} API keys per account");
    }

    let secret = generate_key();
    let id = Uuid::new_v4().to_string();
    let prefix = secret.chars().take(12).collect::<String>();
    let key_hash = hash_key(&secret);
    let name = name.trim();

    sqlx::query(
        "INSERT INTO api_keys (id, owner_uid, name, prefix, key_hash) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(uid)
    .bind(name)
    .bind(&prefix)
    .bind(&key_hash)
    .execute(db)
    .await?;

    let created_at: String =
        sqlx::query_scalar("SELECT created_at FROM api_keys WHERE id = ?")
            .bind(&id)
            .fetch_one(db)
            .await?;

    Ok(MintApiKeyResponse {
        key: ApiKeyPublic {
            id,
            name: name.to_string(),
            prefix,
            created_at,
            last_used_at: None,
        },
        secret,
    })
}

pub async fn revoke_key(db: &SqlitePool, uid: &str, id: &str) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM api_keys WHERE id = ? AND owner_uid = ?")
        .bind(id)
        .bind(uid)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn verify_key(db: &SqlitePool, key: &str) -> Option<String> {
    if !key.starts_with(KEY_PREFIX) || key.len() < 20 {
        return None;
    }
    let key_hash = hash_key(key);
    let uid: Option<String> =
        sqlx::query_scalar("SELECT owner_uid FROM api_keys WHERE key_hash = ?")
            .bind(&key_hash)
            .fetch_optional(db)
            .await
            .ok()?;
    let uid = uid?;
    let _ = sqlx::query("UPDATE api_keys SET last_used_at = datetime('now') WHERE key_hash = ?")
        .bind(&key_hash)
        .execute(db)
        .await;
    Some(uid)
}
