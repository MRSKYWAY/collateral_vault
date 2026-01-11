use anchor_spl::{
    associated_token,
    token_interface::{
        self,
        Mint,
        TokenAccount,
        TokenInterface,
        TransferChecked,
    },
};
use anchor_lang::prelude::log;
use anchor_lang::prelude::InterfaceAccount;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
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
        // Create associated token account for the vault PDA first (uses immutable AccountInfo)
        associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.user.to_account_info(),
                    associated_token: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            ),
        )?;

        // Now mutably borrow and set fields
        let vault = &mut ctx.accounts.vault;

        // Set authority and token account
        vault.owner = ctx.accounts.user.key();
        vault.token_account = ctx.accounts.vault_token_account.key();

        // Genesis invariants
        vault.total_balance = 0;
        vault.locked_balance = 0;
        vault.available_balance = 0;

        vault.total_deposited = 0;
        vault.total_withdrawn = 0;

        vault.created_at = Clock::get()?.unix_timestamp;
        vault.bump = ctx.bumps.vault;

        Ok(())
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let vault_key = ctx.accounts.vault.key();
        let user_key = ctx.accounts.user.key();
        let now = Clock::get()?.unix_timestamp;

        let vault = &mut ctx.accounts.vault;

        // SPL transfer (checked)
        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.mint.decimals,
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

        // Extra check for full withdrawal: no open positions (locked == 0)
        if amount == vault.total_balance {
            require!(vault.locked_balance == 0, VaultError::OpenPositionsExist);
        }

        // PDA signer seeds
        let seeds = &[
            b"vault",
            vault.owner.as_ref(),
            &[vault.bump],
        ];
        let signer = &[&seeds[..]];

        // SPL token transfer: vault → user (checked)
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: vault_ai,
                    mint: ctx.accounts.mint.to_account_info(),
                },
                signer,
            ),
            amount,
            ctx.accounts.mint.decimals,
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

    pub fn initialize_vault_authority(
        ctx: Context<InitializeVaultAuthority>,
        authorized_programs: Vec<Pubkey>,
    ) -> Result<()> {
        require!(
            authorized_programs.len() <= 16,
            VaultError::InvalidAmount
        );

        let authority = &mut ctx.accounts.vault_authority;
        authority.authorized_programs = authorized_programs;
        authority.bump = ctx.bumps.vault_authority;

        Ok(())
    }

    pub fn lock_collateral(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let vault = &mut ctx.accounts.vault;
        let caller_key = ctx.accounts.caller_program.key();
        let vault_key = vault.key();
        let now = Clock::get()?.unix_timestamp;

        require!(
            vault.available_balance >= amount,
            VaultError::InsufficientAvailableBalance
        );

        vault.available_balance = vault
            .available_balance
            .checked_sub(amount)
            .ok_or(VaultError::MathOverflow)?;

        vault.locked_balance = vault
            .locked_balance
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        emit!(LockEvent {
            vault: vault_key,
            caller: caller_key,
            amount,
            new_locked_balance: vault.locked_balance,
            timestamp: now,
        });

        Ok(())
    }

    pub fn unlock_collateral(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let vault = &mut ctx.accounts.vault;
        let caller_key = ctx.accounts.caller_program.key();
        let vault_key = vault.key();
        let now = Clock::get()?.unix_timestamp;

        require!(
            vault.locked_balance >= amount,
            VaultError::InsufficientAvailableBalance
        );

        vault.locked_balance = vault
            .locked_balance
            .checked_sub(amount)
            .ok_or(VaultError::MathOverflow)?;

        vault.available_balance = vault
            .available_balance
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        emit!(UnlockEvent {
            vault: vault_key,
            caller: caller_key,
            amount,
            new_locked_balance: vault.locked_balance,
            timestamp: now,
        });

        Ok(())
    }

    pub fn transfer_collateral(ctx: Context<TransferCollateral>, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let from_vault = &mut ctx.accounts.from_vault;
        let to_vault = &mut ctx.accounts.to_vault;
        let now = Clock::get()?.unix_timestamp;

        require!(
            from_vault.total_balance >= amount,
            VaultError::InsufficientAvailableBalance
        );

        from_vault.total_balance = from_vault
            .total_balance
            .checked_sub(amount)
            .ok_or(VaultError::MathOverflow)?;

        from_vault.available_balance = from_vault
            .available_balance
            .checked_sub(amount)
            .ok_or(VaultError::MathOverflow)?;

        to_vault.total_balance = to_vault
            .total_balance
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        to_vault.available_balance = to_vault
            .available_balance
            .checked_add(amount)
            .ok_or(VaultError::MathOverflow)?;

        emit!(TransferEvent {
            from_vault: from_vault.key(),
            to_vault: to_vault.key(),
            amount,
            timestamp: now,
        });

        Ok(())
    }

    pub fn demo_lock(ctx: Context<LockCollateral>, amount: u64) -> Result<()> {
        lock_collateral(ctx, amount)
    }

    pub fn demo_unlock(ctx: Context<UnlockCollateral>, amount: u64) -> Result<()> {
        unlock_collateral(ctx, amount)
    }

    pub fn demo_transfer_collateral(
        ctx: Context<TransferCollateral>,
        amount: u64,
    ) -> Result<()> {
        transfer_collateral(ctx, amount)
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

    /// CHECK: SPL token account that will hold USDT. Created via CPI.
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,  // USDT mint

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>,

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
        constraint = vault_token_account.key() == vault.token_account,
        constraint = vault_token_account.mint == mint.key(),
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        constraint = user_token_account.mint == mint.key(),
    )]
    pub mint: InterfaceAccount<'info, Mint>,

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
        constraint = vault_token_account.key() == vault.token_account,
        constraint = vault_token_account.mint == mint.key(),
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_token_account.mint == mint.key(),
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        constraint = user_token_account.mint == mint.key(),
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct InitializeVaultAuthority<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = VaultAuthority::LEN,
        seeds = [b"vault_authority"],
        bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LockCollateral<'info> {
    /// CHECK: caller program (CPI)
    pub caller_program: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump,
        constraint = vault_authority
            .authorized_programs
            .contains(&caller_program.key()),
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,
}

#[derive(Accounts)]
pub struct UnlockCollateral<'info> {
    /// CHECK: caller program (CPI)
    pub caller_program: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump,
        constraint = vault_authority
            .authorized_programs
            .contains(&caller_program.key()),
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, CollateralVault>,
}

#[derive(Accounts)]
pub struct TransferCollateral<'info> {
    /// CHECK: calling program
    pub caller_program: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority.bump,
        constraint = vault_authority
            .authorized_programs
            .contains(&caller_program.key()),
    )]
    pub vault_authority: Account<'info, VaultAuthority>,

    #[account(
        mut,
        seeds = [b"vault", from_vault.owner.as_ref()],
        bump = from_vault.bump,
    )]
    pub from_vault: Account<'info, CollateralVault>,

    #[account(
        mut,
        seeds = [b"vault", to_vault.owner.as_ref()],
        bump = to_vault.bump,
    )]
    pub to_vault: Account<'info, CollateralVault>,
}

#[cfg(test)]
mod tests;