mod config;
mod solana;
mod db;
mod models;
mod routes;
mod tx;

use axum::{Router, routing::get};
use axum::routing::post;
use routes::{health, vault};
use tokio::net::TcpListener;
use crate::db::init_db;
use routes::vault::{
    get_vault,
    get_balance,
    // get_transactions,
    // get_tvl,
    tx_deposit,
    tx_withdraw,
    tx_lock,
    tx_unlock,
    tx_transfer,
};

// use routes::vault::build_deposit;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = init_db().await;
    let state = AppState { db };
    println!("🗄️ Database initialized");

    let app = Router::new()
        .route("/health", get(health::health))
        .route("/vault/:owner", get(vault::get_vault))
        .route("/vault/:owner/balance", get(vault::get_balance))
        .route("/vault/:owner/transactions", get(transactions::get_transactions))
        .route("/tvl", get(transactions::get_tvl))
        .route("/tx/deposit", post(tx_deposit))
        .route("/tx/withdraw", post(tx_withdraw))
        .route("/tx/lock", post(tx_lock))
        .route("/tx/unlock", post(tx_unlock))
        .route("/tx/transfer", post(tx_transfer))
        .with_state(state)
;


    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Backend running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
