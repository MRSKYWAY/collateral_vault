use axum::{extract::Path, Json};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::models::{AmountRequest, TxResponse};
use crate::tx::build_deposit_intent;
use base64::{engine::general_purpose, Engine as _};
use crate::models::VaultResponse;
use crate::solana::fetch_vault;

pub async fn get_vault(Path(owner): Path<String>) -> Json<VaultResponse> {
    let owner_pubkey = Pubkey::from_str(&owner)
        .expect("invalid pubkey");

    let (vault_pda, vault) =
        fetch_vault(&owner_pubkey).expect("vault fetch failed");

    Json(VaultResponse {
        owner: owner_pubkey.to_string(),
        vault_pda: vault_pda.to_string(),
        total_balance: vault.total_balance,
        locked_balance: vault.locked_balance,
        available_balance: vault.available_balance,
    })
}

pub async fn get_balance(Path(owner): Path<String>) -> Json<VaultResponse> {
    get_vault(Path(owner)).await
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

pub async fn deposit_intent(
    Json(req): Json<AmountRequest>,
) -> Json<serde_json::Value> {
    let intent = build_deposit_intent(req.amount);

    Json(serde_json::json!({
        "program": "collateral_vault",
        "intent": intent,
        "note": "Client should build and sign the Anchor instruction"
    }))
}
use crate::db::{vaults::upsert_vault, now_ts};
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