pub mod health;
pub mod vault;
pub mod transactions;

pub fn now_ts() -> chrono::DateTime<chrono::Utc> {
    use chrono::Utc;
    Utc::now()
}