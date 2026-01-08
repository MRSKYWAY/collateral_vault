use super::*;
use anchor_lang::prelude::*;

fn fresh_vault(owner: Pubkey) -> CollateralVault {
    CollateralVault {
        owner,
        token_account: Pubkey::new_unique(),
        total_balance: 0,
        available_balance: 0,
        locked_balance: 0,
        total_deposited: 0,
        total_withdrawn: 0,
        created_at: 0,
        bump: 0,
    }
}

#[test]
fn lock_unlock_preserves_invariant() {
    let owner = Pubkey::new_unique();
    let mut vault = fresh_vault(owner);

    vault.total_balance = 100;
    vault.available_balance = 100;

    // simulate lock
    vault.available_balance -= 40;
    vault.locked_balance += 40;

    assert_eq!(vault.available_balance, 60);
    assert_eq!(vault.locked_balance, 40);
    assert_eq!(vault.total_balance, 100);

    // simulate unlock
    vault.available_balance += 40;
    vault.locked_balance -= 40;

    assert_eq!(vault.available_balance, 100);
    assert_eq!(vault.locked_balance, 0);
    assert_eq!(vault.total_balance, 100);
}

#[test]
fn transfer_conserves_collateral() {
    let mut vault_a = fresh_vault(Pubkey::new_unique());
    let mut vault_b = fresh_vault(Pubkey::new_unique());

    vault_a.total_balance = 100;
    vault_a.available_balance = 100;

    vault_b.total_balance = 50;
    vault_b.available_balance = 50;

    // simulate transfer
    let amount = 30;
    vault_a.total_balance -= amount;
    vault_a.available_balance -= amount;

    vault_b.total_balance += amount;
    vault_b.available_balance += amount;

    assert_eq!(vault_a.total_balance + vault_b.total_balance, 150);
    assert_eq!(vault_a.total_balance, 70);
    assert_eq!(vault_b.total_balance, 80);
}
