

#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use collateral_vault::state::CollateralVault; // Adjust path if needed

#[derive(Arbitrary, Debug)]
struct BalanceOp {
    deposit: u64,
    lock_amount: u64,
    unlock_amount: u64,
    withdraw: u64,
}

fuzz_target!(|ops: Vec<BalanceOp>| {
    let mut vault = CollateralVault {
        total_balance: 0,
        locked_balance: 0,
        available_balance: 0,
        // ... other fields default
        owner: Pubkey::new_unique(),
        token_account: Pubkey::new_unique(),
        total_deposited: 0,
        total_withdrawn: 0,
        created_at: 0,
        bump: 0,
    };

    for op in ops {
        // Simulate deposit
        if let Some(new_total) = vault.total_balance.checked_add(op.deposit) {
            if let Some(new_avail) = vault.available_balance.checked_add(op.deposit) {
                vault.total_balance = new_total;
                vault.available_balance = new_avail;
                vault.total_deposited += op.deposit;
            }
        }

        // Simulate lock
        if op.lock_amount <= vault.available_balance {
            if let Some(new_locked) = vault.locked_balance.checked_add(op.lock_amount) {
                if let Some(new_avail) = vault.available_balance.checked_sub(op.lock_amount) {
                    vault.locked_balance = new_locked;
                    vault.available_balance = new_avail;
                }
            }
        }

        // Simulate unlock
        if op.unlock_amount <= vault.locked_balance {
            if let Some(new_locked) = vault.locked_balance.checked_sub(op.unlock_amount) {
                if let Some(new_avail) = vault.available_balance.checked_add(op.unlock_amount) {
                    vault.locked_balance = new_locked;
                    vault.available_balance = new_avail;
                }
            }
        }

        // Simulate withdraw
        if op.withdraw <= vault.available_balance {
            if let Some(new_total) = vault.total_balance.checked_sub(op.withdraw) {
                if let Some(new_avail) = vault.available_balance.checked_sub(op.withdraw) {
                    vault.total_balance = new_total;
                    vault.available_balance = new_avail;
                    vault.total_withdrawn += op.withdraw;
                }
            }
        }

        // Invariants
        assert_eq!(vault.available_balance, vault.total_balance - vault.locked_balance);
        assert!(vault.total_balance <= vault.total_deposited - vault.total_withdrawn);
        assert!(vault.locked_balance <= vault.total_balance);
        assert!(vault.available_balance <= vault.total_balance);
    }
});

// Run with: cargo fuzz run balances -- -runs=1000000 (or more)

// For other security:
// Run in terminal:
// cargo audit
// cargo clippy --all-targets --all-features -- -D warnings
// For backend: Add auth middleware if not present, e.g., in src/main.rs use axum::middleware for JWT on sensitive routes.