use anyhow::{Result, anyhow};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use borsh::BorshDeserialize;

use crate::models::CollateralVaultAccount;

pub const RPC_URL: &str = "http://127.0.0.1:8899";
pub const PROGRAM_ID: &str = "CqYzY3dRdbEBUg29TFBWXLrQQhumMeyRr6vJv76RNiTq";

pub fn fetch_vault(owner: &Pubkey) -> Result<(Pubkey, CollateralVaultAccount)> {
    let client = RpcClient::new(RPC_URL.to_string());
    let program_id = Pubkey::from_str(PROGRAM_ID)?;

    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref()],
        &program_id,
    );

    // Add simple retry
    let mut attempts = 0;
    loop {
        match client.get_account(&vault_pda) {
            Ok(account) => {
                let data = &account.data[8..];
                let vault = CollateralVaultAccount::try_from_slice(data)?;
                return Ok((vault_pda, vault));
            }
            Err(_) if attempts < 3 => {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Err(_) => return Err(anyhow!("vault account not found after retries")),
        }
    }
}