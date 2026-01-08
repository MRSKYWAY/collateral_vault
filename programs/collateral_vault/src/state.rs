use anchor_lang::prelude::*;

#[account]
pub struct CollateralVault {
    /// Owner of the vault (user wallet)
    pub owner: Pubkey,

    /// SPL token account holding USDT, owned by the vault PDA
    pub token_account: Pubkey,

    /// Total collateral in vault
    pub total_balance: u64,

    /// Collateral locked for positions
    pub locked_balance: u64,

    /// Collateral available for withdrawal
    pub available_balance: u64,

    /// Lifetime deposited amount
    pub total_deposited: u64,

    /// Lifetime withdrawn amount
    pub total_withdrawn: u64,

    /// Vault creation timestamp
    pub created_at: i64,

    /// PDA bump
    pub bump: u8,
}

impl CollateralVault {
    pub const LEN: usize =
        8 +   // discriminator
        32 +  // owner
        32 +  // token_account
        8 +   // total_balance
        8 +   // locked_balance
        8 +   // available_balance
        8 +   // total_deposited
        8 +   // total_withdrawn
        8 +   // created_at
        1;    // bump
}


#[account]
pub struct VaultAuthority {
    pub authorized_programs: Vec<Pubkey>,
    pub bump: u8,
}

impl VaultAuthority {
    pub const LEN: usize =
        8 +   // discriminator
        4 +   // vec length
        (32 * 16) + // up to 16 authorized programs (reasonable cap)
        1;    // bump
}
