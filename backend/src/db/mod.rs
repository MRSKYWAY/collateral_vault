use sqlx::{PgPool, postgres::PgPoolOptions};
use chrono::Utc;

pub mod vaults;
pub use vaults::upsert_vault;

pub async fn init_db() -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(crate::config::DATABASE_URL)
        .await
        .expect("DB init failed");

    let schema = include_str!("schema.sql");
    for query_str in schema.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(&format!("{};", query_str.trim()))
            .execute(&pool)
            .await
            .expect(&format!("Failed to execute: {}", query_str));
    }

    println!("🗄️ Database initialized");
    pool
}

pub fn now_ts() -> chrono::DateTime<Utc> {
    Utc::now()
}