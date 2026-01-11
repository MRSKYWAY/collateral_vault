use sqlx::PgPool;
use chrono::{DateTime, Utc};

pub async fn upsert_vault(
    pool: &PgPool,
    owner: &str,
    vault_pda: &str,
    total: u64,
    locked: u64,
    available: u64,
    ts: DateTime<Utc>,
) {
    sqlx::query(
    r#"
    INSERT INTO vaults (owner, vault_pda, total_balance, locked_balance, available_balance, last_updated)
    VALUES ($1, $2, $3, $4, $5, $6)
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