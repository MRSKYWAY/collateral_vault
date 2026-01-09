use axum::{extract::Path, Json};
use crate::AppState;

pub async fn get_transactions(
    state: axum::extract::State<AppState>,
    Path(owner): Path<String>,
) -> Json<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        "SELECT tx_type, amount, timestamp FROM vault_transactions WHERE owner = ?",
        owner
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    Json(rows.into_iter().map(|r| {
        serde_json::json!({
            "type": r.tx_type,
            "amount": r.amount,
            "timestamp": r.timestamp
        })
    }).collect())
}

pub async fn get_tvl(
    state: axum::extract::State<AppState>,
) -> Json<i64> {
    let row = sqlx::query!(
        "SELECT SUM(total_balance) as tvl FROM vaults"
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(row.tvl.unwrap_or(0))
}
