use anchor_spl::token_interface::{
    self,
    TokenAccount,
    TokenInterface,
    Transfer,
};
use anchor_lang::prelude::InterfaceAccount;
use anchor_lang::prelude::*;

pub mod state;
pub mod error;
pub mod events;

use state::*;
use error::*;
use events::*;


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


pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let vault_key = ctx.accounts.vault.key();
    let user_key = ctx.accounts.user.key();
    let now = Clock::get()?.unix_timestamp;

    let vault = &mut ctx.accounts.vault;

    // SPL transfer
    token_interface::transfer(
    CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    ),
    amount,
)?;


    vault.total_balance = vault
        .total_balance
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;

    vault.available_balance = vault
        .available_balance
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;

    vault.total_deposited = vault
        .total_deposited
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;

    emit!(DepositEvent {
        user: user_key,
        vault: vault_key,
        amount,
        new_total_balance: vault.total_balance,
        timestamp: now,
    });

    Ok(())
}

pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::InvalidAmount);

    let vault_key = ctx.accounts.vault.key();
    let vault_ai = ctx.accounts.vault.to_account_info();
    let user_key = ctx.accounts.user.key();
    let now = Clock::get()?.unix_timestamp;

    let vault = &mut ctx.accounts.vault;

    // Enforce available balance (locked funds cannot be withdrawn)
    require!(
        vault.available_balance >= amount,
        VaultError::InsufficientAvailableBalance
    );

    // PDA signer seeds
    let seeds = &[
        b"vault",
        vault.owner.as_ref(),
        &[vault.bump],
    ];
    let signer = &[&seeds[..]];

    // SPL token transfer: vault → user
    token_interface::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: vault_ai,
            },
            signer,
        ),
        amount,
    )?;

    // Update balances (checked math)
    vault.total_balance = vault
        .total_balance
        .checked_sub(amount)
        .ok_or(VaultError::MathOverflow)?;

    vault.available_balance = vault
        .available_balance
        .checked_sub(amount)
        .ok_or(VaultError::MathOverflow)?;

    vault.total_withdrawn = vault
        .total_withdrawn
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;

    emit!(WithdrawEvent {
        user: user_key,
        vault: vault_key,
        amount,
        new_total_balance: vault.total_balance,
        timestamp: now,
    });

    Ok(())
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

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
        constraint = vault.owner == user.key(),
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.key() == vault.token_account
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
        constraint = vault.owner == user.key(),
    )]
    pub vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        constraint = vault_token_account.key() == vault.token_account
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}




