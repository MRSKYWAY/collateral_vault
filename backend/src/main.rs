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
// use routes::vault::build_deposit;
use routes::vault::deposit_intent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _db = init_db().await?;
    println!("🗄️ Database initialized");

    let app = Router::new()
        .route("/health", get(health::health))
        .route("/vault/:owner", get(vault::get_vault))
        .route("/vault/:owner/balance", get(vault::get_balance))
        .route("/tx/deposit", post(deposit_intent))
;


    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Backend running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
