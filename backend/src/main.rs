mod config;
mod solana;
mod db;
mod models;
mod routes;
mod tx;

use axum::{Router, routing::{get, post}};
use routes::health::health;
use routes::vault::{get_vault, get_balance, tx_deposit, tx_withdraw, tx_lock, tx_unlock, tx_transfer, confirm_tx};
use routes::transactions::{get_transactions, get_tvl};
use tokio::net::TcpListener;
use crate::db::init_db;
use axum::extract::{State, ws::{WebSocketUpgrade, WebSocket, Message}};
use tokio::time::{sleep, Duration};
use sqlx::PgPool;
use sqlx::Row;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use chrono::Utc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = init_db().await;
    let state = AppState { db: db.clone() };
    println!("🗄️ Database initialized");

    let db_clone = db.clone();
    tokio::spawn(async move {
        loop {
            reconcile_all_vaults(&db_clone).await;
            sleep(Duration::from_secs(60)).await;
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/vault/:owner", get(get_vault))
        .route("/vault/:owner/balance", get(get_balance))
        .route("/vault/:owner/transactions", get(get_transactions))
        .route("/tvl", get(get_tvl))
        .route("/tx/deposit", post(tx_deposit))
        .route("/tx/withdraw", post(tx_withdraw))
        .route("/tx/lock", post(tx_lock))
        .route("/tx/unlock", post(tx_unlock))
        .route("/tx/transfer", post(tx_transfer))
        .route("/tx/confirm", post(confirm_tx))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Backend running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn reconcile_all_vaults(db: &PgPool) {
    let owners: Vec<String> = sqlx::query("SELECT owner FROM vaults")
        .fetch_all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>(0))
        .collect();

    for owner in owners {
        let owner_pk = Pubkey::from_str(&owner).unwrap();
        if let Ok((_, vault)) = crate::solana::fetch_vault(&owner_pk) {
            let db_row = sqlx::query("SELECT total_balance, locked_balance, available_balance FROM vaults WHERE owner = $1")
                .bind(&owner)
                .fetch_one(db)
                .await
                .unwrap();

            let db_total = db_row.get::<i64, _>(0) as u64;
            let db_locked = db_row.get::<i64, _>(1) as u64;
            let db_available = db_row.get::<i64, _>(2) as u64;

            if vault.total_balance != db_total || vault.locked_balance != db_locked || vault.available_balance != db_available {
                sqlx::query("INSERT INTO reconciliation_logs (vault_owner, discrepancy, logged_at) VALUES ($1, $2, NOW())")
                    .bind(&owner)
                    .bind(format!("Mismatch: on-chain {} vs DB {}", vault.total_balance, db_total))
                    .execute(db)
                    .await
                    .unwrap();
            } else {
                sqlx::query("INSERT INTO balance_snapshots (vault_owner, total_balance, locked_balance, available_balance, snapshot_at) VALUES ($1, $2, $3, $4, NOW())")
                    .bind(&owner)
                    .bind(vault.total_balance as i64)
                    .bind(vault.locked_balance as i64)
                    .bind(vault.available_balance as i64)
                    .execute(db)
                    .await
                    .unwrap();
            }
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, state: State<AppState>) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_ws(socket, state.db.clone()))
}

async fn handle_ws(mut socket: WebSocket, db: PgPool) {
    loop {
        let tvl_row = sqlx::query("SELECT SUM(total_balance) as tvl FROM vaults")
            .fetch_one(&db)
            .await
            .unwrap();
        let tvl = tvl_row.get::<Option<i64>, _>("tvl").unwrap_or(0);

        if socket.send(Message::Text(serde_json::json!({"tvl": tvl}).to_string())).await.is_err() {
            break;
        }
        sleep(Duration::from_secs(5)).await;
    }
}