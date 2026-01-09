use axum::{extract::Path, Json};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::models::{AmountRequest, TransferRequest};
use crate::tx::*;
use crate::models::VaultResponse;
use crate::solana::fetch_vault;
use crate::models::IntentResponse;
use crate::db::{upsert_vault, now_ts};
use crate::AppState;

pub async fn get_vault(
    state: axum::extract::State<AppState>,
    Path(owner): Path<String>,
) -> Json<VaultResponse> {
    let owner_pk = Pubkey::from_str(&owner).unwrap();
    let (vault_pda, vault) = fetch_vault(&owner_pk).unwrap();

    upsert_vault(
        &state.db,
        &owner,
        &vault_pda.to_string(),
        vault.total_balance,
        vault.locked_balance,
        vault.available_balance,
        now_ts(),
    ).await;

    Json(VaultResponse {
        owner,
        vault_pda: vault_pda.to_string(),
        total_balance: vault.total_balance,
        locked_balance: vault.locked_balance,
        available_balance: vault.available_balance,
    })
}

pub async fn get_balance(
    state: axum::extract::State<AppState>,
    Path(owner): Path<String>,
) -> Json<VaultResponse> {
    get_vault(state, Path(owner)).await
}


// pub async fn build_deposit(
//     Json(req): Json<AmountRequest>,
// ) -> Json<TxResponse> {
//     let owner = Pubkey::from_str(&req.owner).expect("invalid pubkey");

//     let tx = build_deposit_tx(&owner, req.amount)
//         .expect("tx build failed");

//     let serialized = bincode::serialize(&tx).unwrap();
//     let encoded = general_purpose::STANDARD.encode(serialized);

//     Json(TxResponse {
//         transaction_base64: encoded,
//     })
// }



pub async fn tx_deposit(Json(req): Json<AmountRequest>) -> Json<IntentResponse> {
    Json(deposit_intent(req.amount))
}

pub async fn tx_withdraw(Json(req): Json<AmountRequest>) -> Json<IntentResponse> {
    Json(withdraw_intent(req.amount))
}

pub async fn tx_lock(Json(req): Json<AmountRequest>) -> Json<IntentResponse> {
    Json(lock_intent(req.amount))
}

pub async fn tx_unlock(Json(req): Json<AmountRequest>) -> Json<IntentResponse> {
    Json(unlock_intent(req.amount))
}

pub async fn tx_transfer(Json(req): Json<TransferRequest>) -> Json<IntentResponse> {
    Json(transfer_intent(&req.from, &req.to, req.amount))
}
