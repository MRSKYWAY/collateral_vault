use axum::{extract::{Path, State, Json}, Json as AxumJson, response::IntoResponse};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use crate::models::{AmountRequest, TransferRequest, TxResponse, ConfirmRequest, VaultResponse, IntentResponse};
use crate::solana::fetch_vault;
use crate::db::upsert_vault;
use crate::AppState;
use solana_sdk::{instruction::{Instruction, AccountMeta}, hash::hashv, pubkey};
use base64::{engine::general_purpose, Engine};
use bincode;
use solana_client::rpc_client::RpcClient;
use crate::solana::{RPC_URL, PROGRAM_ID};
use crate::tx::{deposit_intent, withdraw_intent, lock_intent, unlock_intent, transfer_intent};
use chrono::Utc;
use spl_token::id as token_program_id;

// Assume USDT mint (replace with real)
const USDT_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // Devnet USDT example

pub async fn get_vault(
    State(state): State<AppState>,
    Path(owner): Path<String>,
) -> AxumJson<VaultResponse> {
    let owner_pk = Pubkey::from_str(&owner).expect("Invalid pubkey");
    let (vault_pda, vault) = fetch_vault(&owner_pk).expect("Vault fetch failed");

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

fn instruction_discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", name);
    let mut d = [0u8; 8];
    d.copy_from_slice(&hashv(&[preimage.as_bytes()]).to_bytes()[0..8]);
    d
}

// pub async fn tx_deposit(
//     Json(req): Json<AmountRequest>,
// ) -> impl IntoResponse {
//     let owner = Pubkey::from_str(&req.owner).expect("Invalid pubkey");
//     let program_id = Pubkey::from_str(PROGRAM_ID).expect("Invalid program ID");

//     let client = RpcClient::new(RPC_URL.to_string());
//     let blockhash = client.get_latest_blockhash().expect("Blockhash failed");

//     let discriminator = instruction_discriminator("deposit");
//     let mut data = discriminator.to_vec();
//     data.extend_from_slice(&req.amount.to_le_bytes());

//     let (vault_pda, _) = Pubkey::find_program_address(&[b"vault", owner.as_ref()], &program_id);

//     // User ATA (assume exists; fetch or create if needed)
//     let user_token = spl_associated_token_account::get_associated_token_address(&owner, &pubkey::Pubkey::from_str(USDT_MINT).unwrap());
//     let vault_token = spl_associated_token_account::get_associated_token_address(&vault_pda, &pubkey::Pubkey::from_str(USDT_MINT).unwrap());

//     let ix = Instruction {
//         program_id,
//         accounts: vec![
//             AccountMeta::new(owner, true),  // user (signer)
//             AccountMeta::new(vault_pda, false),  // vault
//             AccountMeta::new(user_token, false),  // user_token_account
//             AccountMeta::new(vault_token, false),  // vault_token_account
//             AccountMeta::new_readonly(token_program_id(), false),  // token_program
//             AccountMeta::new_readonly(pubkey::Pubkey::from_str(USDT_MINT).unwrap(), false),  // mint
//         ],
//         data,
//     };

//     let message = VersionedMessage::Legacy(Message::new(&[ix], Some(&owner)));
//     let tx = Transaction::new_unsigned(message);

//     let serialized = bincode::serialize(&tx).expect("Serialize failed");
//     let encoded = general_purpose::STANDARD.encode(serialized);

//     AxumJson(TxResponse {
//         transaction_base64: encoded,
//     })
// }

// pub async fn tx_withdraw(
//     Json(req): Json<AmountRequest>,
// ) -> impl IntoResponse {
//     let owner = Pubkey::from_str(&req.owner).expect("Invalid pubkey");
//     let program_id = Pubkey::from_str(PROGRAM_ID).expect("Invalid program ID");

//     let client = RpcClient::new(RPC_URL.to_string());
//     let blockhash = client.get_latest_blockhash().expect("Blockhash failed");

//     let discriminator = instruction_discriminator("withdraw");
//     let mut data = discriminator.to_vec();
//     data.extend_from_slice(&req.amount.to_le_bytes());

//     let (vault_pda, _) = Pubkey::find_program_address(&[b"vault", owner.as_ref()], &program_id);

//     let user_token = spl_associated_token_account::get_associated_token_address(&owner, &pubkey::Pubkey::from_str(USDT_MINT).unwrap());
//     let vault_token = spl_associated_token_account::get_associated_token_address(&vault_pda, &pubkey::Pubkey::from_str(USDT_MINT).unwrap());

//     let ix = Instruction {
//         program_id,
//         accounts: vec![
//             AccountMeta::new(owner, true),  // user (signer)
//             AccountMeta::new(vault_pda, false),  // vault
//             AccountMeta::new(vault_token, false),  // vault_token_account
//             AccountMeta::new(user_token, false),  // user_token_account
//             AccountMeta::new_readonly(token_program_id(), false),  // token_program
//             AccountMeta::new_readonly(pubkey::Pubkey::from_str(USDT_MINT).unwrap(), false),  // mint
//         ],
//         data,
//     };

//     let message = VersionedMessage::Legacy(Message::new(&[ix], Some(&owner)));
//     let tx = Transaction::new_unsigned(message);

//     let serialized = bincode::serialize(&tx).expect("Serialize failed");
//     let encoded = general_purpose::STANDARD.encode(serialized);

//     AxumJson(TxResponse {
//         transaction_base64: encoded,
//     })
// }

pub async fn tx_deposit(
    Json(req): Json<AmountRequest>,
) -> Json<IntentResponse> {
    Json(deposit_intent(req.amount))
}

pub async fn tx_withdraw(
    Json(req): Json<AmountRequest>,
) -> Json<IntentResponse> {
    Json(withdraw_intent(req.amount))
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
    .expect("Insert failed");

    AxumJson("Transaction logged".to_string())
}

// Bonus: Mock position open (simulates CPI lock)
pub async fn mock_position_open(
    Json(req): Json<AmountRequest>,
) -> AxumJson<String> {
    // Simulate call to lock_collateral via RPC (use demo_lock if available)
    // For real: Build tx for demo_lock, sign as admin, submit
    AxumJson(format!("Mock locked {} for owner {}", req.amount, req.owner))
}