use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod vaults;
pub use vaults::upsert_vault;

pub async fn init_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .expect("DB init failed");

    sqlx::query(include_str!("schema.sql"))
        .execute(&pool)
        .await
        .expect("Schema load failed");

    println!("🗄️ Database initialized");
    pool
}

pub fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
