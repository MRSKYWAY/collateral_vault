use axum::{extract::{Path, State, Json}, Json as AxumJson};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::models::{AmountRequest, TransferRequest, TxResponse, ConfirmRequest, VaultResponse, IntentResponse};
use crate::solana::fetch_vault;
use crate::db::{upsert_vault};
use crate::AppState;
use solana_sdk::{instruction::{Instruction, AccountMeta}, transaction::Transaction, message::Message};
use base64::{engine::general_purpose, Engine};
use bincode;
use solana_client::rpc_client::RpcClient;
use crate::solana::{RPC_URL, PROGRAM_ID};
use crate::tx::{lock_intent, unlock_intent, transfer_intent};
use chrono::Utc;
use spl_token;
use axum::response::IntoResponse;
pub async fn get_vault(
    State(state): State<AppState>,
    Path(owner): Path<String>,
) -> AxumJson<VaultResponse> {
    let owner_pk = Pubkey::from_str(&owner).unwrap();
    let (vault_pda, vault) = fetch_vault(&owner_pk).unwrap();

    upsert_vault(
        &state.db,
        &owner,
        &vault_pda.to_string(),
        vault.total_balance,
        vault.locked_balance,
        vault.available_balance,
        Utc::now(),
    ).await;

    AxumJson(VaultResponse {
        owner,
        vault_pda: vault_pda.to_string(),
        total_balance: vault.total_balance,
        locked_balance: vault.locked_balance,
        available_balance: vault.available_balance,
    })
}

pub async fn get_balance(
    State(state): State<AppState>,
    Path(owner): Path<String>,
) -> AxumJson<VaultResponse> {
    get_vault(State(state), Path(owner)).await
}

pub async fn tx_deposit(
    Json(req): Json<AmountRequest>,
) -> impl IntoResponse {
    let owner = Pubkey::from_str(&req.owner).expect("invalid pubkey");
    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();

    let client = RpcClient::new(RPC_URL.to_string());
    let blockhash = client.get_latest_blockhash().unwrap();

    let discriminator = [0, 0, 0, 0, 0, 0, 0, 0]; // Placeholder - get from Anchor
    let mut data = Vec::new();
    data.extend_from_slice(&discriminator);
    data.extend_from_slice(&req.amount.to_le_bytes());

    let (vault_pda, _) = Pubkey::find_program_address(&[b"vault", owner.as_ref()], &program_id);

    // Placeholder token accounts - in real, req should include or fetch
    let user_token = Pubkey::new_unique();
    let vault_token = Pubkey::new_unique();
    let token_program = spl_token::id();

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner, true), // user signer
            AccountMeta::new(vault_pda, false), // vault writable
            AccountMeta::new(user_token, false), // user_token_account writable
            AccountMeta::new(vault_token, false), // vault_token_account writable
            AccountMeta::new_readonly(token_program, false), // token_program
        ],
        data,
    };

    let message = Message::new(&[ix], Some(&owner));
    let tx = Transaction::new_unsigned(message);

    let serialized = bincode::serialize(&tx).unwrap();
    let encoded = general_purpose::STANDARD.encode(serialized);

    AxumJson(TxResponse {
        transaction_base64: encoded,
    })
}

pub async fn tx_withdraw(
    Json(req): Json<AmountRequest>,
) -> impl IntoResponse {
    let owner = Pubkey::from_str(&req.owner).expect("invalid pubkey");
    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();

    let client = RpcClient::new(RPC_URL.to_string());
    let blockhash = client.get_latest_blockhash().unwrap();

    let discriminator = [1, 1, 1, 1, 1, 1, 1, 1]; // Placeholder for withdraw
    let mut data = Vec::new();
    data.extend_from_slice(&discriminator);
    data.extend_from_slice(&req.amount.to_le_bytes());

    let (vault_pda, _) = Pubkey::find_program_address(&[b"vault", owner.as_ref()], &program_id);

    let user_token = Pubkey::new_unique();
    let vault_token = Pubkey::new_unique();
    let token_program = spl_token::id();

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(user_token, false),
            AccountMeta::new(vault_token, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data,
    };

    let message = Message::new(&[ix], Some(&owner));
    let tx = Transaction::new_unsigned(message);

    let serialized = bincode::serialize(&tx).unwrap();
    let encoded = general_purpose::STANDARD.encode(serialized);

    AxumJson(TxResponse {
        transaction_base64: encoded,
    })
}

pub async fn tx_lock(Json(req): Json<AmountRequest>) -> AxumJson<IntentResponse> {
    AxumJson(lock_intent(req.amount))
}

pub async fn tx_unlock(Json(req): Json<AmountRequest>) -> AxumJson<IntentResponse> {
    AxumJson(unlock_intent(req.amount))
}

pub async fn tx_transfer(Json(req): Json<TransferRequest>) -> AxumJson<IntentResponse> {
    AxumJson(transfer_intent(&req.from, &req.to, req.amount))
}

pub async fn confirm_tx(
    State(state): State<AppState>,
    Json(req): Json<ConfirmRequest>,
) -> AxumJson<String> {
    sqlx::query(
        "INSERT INTO vault_transactions (owner, tx_type, amount, signature, timestamp) VALUES ($1, $2, $3, $4, NOW())"
    )
    .bind(req.owner)
    .bind(req.event_type)
    .bind(req.amount as i64)
    .bind(req.sig)
    .execute(&state.db)
    .await
    .unwrap();

    AxumJson("Transaction logged".to_string())
}