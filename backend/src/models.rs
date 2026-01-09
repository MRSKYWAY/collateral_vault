use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultResponse {
    pub owner: String,
    pub vault_pda: String,
    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct VaultRow {
    pub id: String,
    pub owner_pubkey: String,
    pub vault_pda: String,
    pub token_account: String,
    pub total_balance: i64,
    pub locked_balance: i64,
    pub available_balance: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultTxRow {
    pub id: String,
    pub vault_pda: String,
    pub tx_signature: String,
    pub event_type: String,
    pub amount: i64,
    pub timestamp: String,
}

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct CollateralVaultAccount {
    pub owner: [u8; 32],
    pub token_account: [u8; 32],

    pub total_balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,

    pub total_deposited: u64,
    pub total_withdrawn: u64,

    pub created_at: i64,
    pub bump: u8,
}

#[derive(Debug, Deserialize)]
pub struct AmountRequest {
    pub owner: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct TxResponse {
    pub transaction_base64: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct IntentResponse {
    pub program: &'static str,
    pub instruction: &'static str,
    pub params: serde_json::Value,
    pub note: &'static str,
}