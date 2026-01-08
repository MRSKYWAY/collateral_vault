use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Insufficient available balance")]
    InsufficientAvailableBalance,

    #[msg("Math overflow")]
    MathOverflow,
}
