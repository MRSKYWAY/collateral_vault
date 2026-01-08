use anchor_lang::prelude::*;

pub mod state;
pub mod error;

use state::*;
use error::*;

declare_id!("CqYzY3dRdbEBUg29TFBWXLrQQhumMeyRr6vJv76RNiTq");

#[program]
pub mod collateral_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // authority
        vault.owner = ctx.accounts.user.key();
        vault.token_account = ctx.accounts.vault_token_account.key();

        // genesis invariants
        vault.total_balance = 0;
        vault.locked_balance = 0;
        vault.available_balance = 0;

        vault.total_deposited = 0;
        vault.total_withdrawn = 0;

        vault.created_at = Clock::get()?.unix_timestamp;
        vault.bump = ctx.bumps.vault;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = CollateralVault::LEN,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, CollateralVault>,

    /// CHECK:
    /// SPL token account that will hold USDT.
    /// Must be owned by the vault PDA (validated in later instructions).
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
