use sqlx::SqlitePool;

pub async fn upsert_vault(
    pool: &SqlitePool,
    owner: &str,
    vault_pda: &str,
    total: u64,
    locked: u64,
    available: u64,
    ts: i64,
) {
    sqlx::query!(
        r#"
        INSERT INTO vaults (owner, vault_pda, total_balance, locked_balance, available_balance, last_updated)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(owner) DO UPDATE SET
            total_balance = excluded.total_balance,
            locked_balance = excluded.locked_balance,
            available_balance = excluded.available_balance,
            last_updated = excluded.last_updated
        "#,
        owner,
        vault_pda,
        total as i64,
        locked as i64,
        available as i64,
        ts
    )
    .execute(pool)
    .await
    .unwrap();
}
