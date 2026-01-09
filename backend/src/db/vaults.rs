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
    sqlx::query(
    r#"
    INSERT INTO vaults (owner, vault_pda, total_balance, locked_balance, available_balance, last_updated)
    VALUES (?, ?, ?, ?, ?, ?)
    ON CONFLICT(owner) DO UPDATE SET
        total_balance = excluded.total_balance,
        locked_balance = excluded.locked_balance,
        available_balance = excluded.available_balance,
        last_updated = excluded.last_updated
    "#
)
.bind(owner)
.bind(vault_pda)
.bind(total as i64)
.bind(locked as i64)
.bind(available as i64)
.bind(ts)
.execute(pool)
.await
.unwrap();

}
