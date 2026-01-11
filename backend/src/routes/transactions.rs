use axum::{extract::{Path, State}, Json};
use crate::AppState;
use sqlx::Row;
use serde_json::json;
use chrono::{DateTime, Utc};

pub async fn get_transactions(
    State(state): State<AppState>,
    Path(owner): Path<String>,
) -> Json<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        "SELECT tx_type, amount, timestamp FROM vault_transactions WHERE owner = $1"
    )
    .bind(owner)
    .fetch_all(&state.db)
    .await
    .unwrap();

    Json(rows.into_iter().map(|r| {
        json!({
            "type": r.get::<String, _>("tx_type"),
            "amount": r.get::<i64, _>("amount"),
            "timestamp": r.get::<DateTime<Utc>, _>("timestamp").to_string()
        })
    }).collect())
}

pub async fn get_tvl(
    State(state): State<AppState>,
) -> Json<i64> {
    let row = sqlx::query(
        "SELECT SUM(total_balance) as tvl FROM vaults"
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(row.get::<Option<i64>, _>("tvl").unwrap_or(0))
}