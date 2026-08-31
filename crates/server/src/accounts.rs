use sqlx::SqlitePool;

pub async fn account_credits(db: &SqlitePool, uid: &str) -> anyhow::Result<i64> {
    ensure_account(db, uid).await?;
    let credits =
        sqlx::query_scalar::<_, i64>("SELECT credits FROM accounts WHERE firebase_uid = ?")
            .bind(uid)
            .fetch_one(db)
            .await?;
    Ok(credits)
}

pub async fn add_credits(db: &SqlitePool, uid: &str, amount: i64) -> anyhow::Result<i64> {
    ensure_account(db, uid).await?;
    sqlx::query("UPDATE accounts SET credits = credits + ? WHERE firebase_uid = ?")
        .bind(amount)
        .bind(uid)
        .execute(db)
        .await?;
    account_credits(db, uid).await
}

pub async fn ensure_account(db: &SqlitePool, uid: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO accounts (firebase_uid, credits) VALUES (?, 0) ON CONFLICT DO NOTHING",
    )
    .bind(uid)
    .execute(db)
    .await?;
    Ok(())
}
