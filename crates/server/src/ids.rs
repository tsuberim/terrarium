use sqlx::SqlitePool;

pub async fn next_global_id(db: &SqlitePool) -> anyhow::Result<u64> {
    let mut tx = db.begin().await?;
    let (next,): (i64,) =
        sqlx::query_as("SELECT next_val FROM id_sequence WHERE name = 'global_id'")
            .fetch_one(&mut *tx)
            .await?;
    sqlx::query("UPDATE id_sequence SET next_val = next_val + 1 WHERE name = 'global_id'")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(next as u64)
}

pub async fn ensure_account_creature_id(db: &SqlitePool, uid: &str) -> anyhow::Result<u64> {
    if let Some((Some(id),)) = sqlx::query_as::<_, (Option<i64>,)>(
        "SELECT account_creature_id FROM accounts WHERE firebase_uid = ?",
    )
    .bind(uid)
    .fetch_optional(db)
    .await?
    {
        return Ok(id as u64);
    }
    let id = next_global_id(db).await?;
    sqlx::query("UPDATE accounts SET account_creature_id = ? WHERE firebase_uid = ?")
        .bind(id as i64)
        .bind(uid)
        .execute(db)
        .await?;
    Ok(id)
}
