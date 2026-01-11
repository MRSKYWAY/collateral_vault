use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
    system_instruction,
    hash::hash,
};
use anchor_lang::prelude::*;
use collateral_vault::{id, instruction, state::CollateralVault};
use spl_token::{
    id as token_id,
    instruction as token_instruction,
    state::{Account as TokenAccount, Mint},
};

#[tokio::test]
async fn test_cpi_deposit_and_lock() {
    let program_id = id();
    let mut test = ProgramTest::new("collateral_vault", program_id, processor!(collateral_vault::entry));
    test.add_program("spl_token", token_id(), None);

    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // Setup keys
    let user = Keypair::new();
    let mint_authority = Keypair::new();
    let mint = Pubkey::new_unique();
    let [vault, _bump] = Pubkey::find_program_address(&[b"vault", user.pubkey().as_ref()], &program_id);
    let user_token = spl_associated_token_account::get_associated_token_address(&user.pubkey(), &mint);
    let vault_token = spl_associated_token_account::get_associated_token_address(&vault, &mint);

    // Fund payer and user
    banks_client.process_transaction(Transaction::new_signed_with_payer(
        &[system_instruction::transfer(&payer.pubkey(), &user.pubkey(), 1_000_000_000)],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    ))
    .await
    .unwrap();

    // Create mint
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let mut tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint,
                mint_rent,
                Mint::LEN as u64,
                &token_id(),
            ),
            token_instruction::initialize_mint(
                &token_id(),
                &mint,
                &mint_authority.pubkey(),
                None,
                6,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Create user token account
    let user_token_rent = rent.minimum_balance(TokenAccount::LEN);
    let mut tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &user_token,
                user_token_rent,
                TokenAccount::LEN as u64,
                &token_id(),
            ),
            token_instruction::initialize_account(
                &token_id(),
                &user_token,
                &mint,
                &user.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Mint tokens to user
    let mut tx = Transaction::new_signed_with_payer(
        &[token_instruction::mint_to(
            &token_id(),
            &mint,
            &user_token,
            &mint_authority.pubkey(),
            &[],
            1000,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[&payer, &mint_authority],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Create vault token account (PDA owned)
    let vault_token_rent = rent.minimum_balance(TokenAccount::LEN);
    let create_vault_token_ix = system_instruction::create_account(
        &payer.pubkey(),
        &vault_token,
        vault_token_rent,
        TokenAccount::LEN as u64,
        &token_id(),
    );
    let init_vault_token_ix = token_instruction::initialize_account(
        &token_id(),
        &vault_token,
        &mint,
        &vault,
    )
    .unwrap();
    let mut tx = Transaction::new_signed_with_payer(
        &[create_vault_token_ix, init_vault_token_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Initialize vault
    let vault_rent = rent.minimum_balance(std::mem::size_of::<CollateralVault>());
    let create_vault_ix = system_instruction::create_account(
        &user.pubkey(),
        &vault,
        vault_rent,
        std::mem::size_of::<CollateralVault>() as u64,
        &program_id,
    );
    let init_ix = instruction::initialize_vault(
        &program_id,
        &vault,
        &vault_token,
        &user.pubkey(),
        &mint,
    )
    .unwrap();
    let mut tx = Transaction::new_signed_with_payer(
        &[create_vault_ix, init_ix],
        Some(&user.pubkey()),
        &[&user],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Deposit
    let deposit_cpi_ix = token_instruction::transfer(
        &token_id(),
        &user_token,
        &vault_token,
        &user.pubkey(),
        &[],
        1000,
    )
    .unwrap();
    let deposit_program_ix = instruction::deposit(
        &program_id,
        &user.pubkey(),
        &vault,
        &user_token,
        &vault_token,
        1000,
    )
    .unwrap();
    let mut tx = Transaction::new_signed_with_payer(
        &[deposit_cpi_ix, deposit_program_ix],
        Some(&user.pubkey()),
        &[&user],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Fetch and assert vault after deposit
    let vault_account = banks_client.get_account(vault).await.unwrap().unwrap();
    let vault_data: CollateralVault = AnchorDeserialize::deserialize(&mut &vault_account.data[..]).unwrap();
    assert_eq!(vault_data.total_balance, 1000);
    assert_eq!(vault_data.available_balance, 1000);
    assert_eq!(vault_data.locked_balance, 0);

    // Simulate lock CPI (assuming no vault authority for simplicity; adjust if needed)
    // Note: If lock requires authority, setup vault_authority PDA similarly
    let mock_caller = Pubkey::new_unique(); // Mock authorized program
    let lock_ix = instruction::lock_collateral(
        &program_id,
        &vault,
        &mock_caller,
        500,
    )
    .unwrap();
    let mut tx = Transaction::new_signed_with_payer(
        &[lock_ix],
        Some(&payer.pubkey()),
        &[&payer],  // Mock signer; in real CPI, the caller program signs
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Fetch and assert after lock
    let vault_account = banks_client.get_account(vault).await.unwrap().unwrap();
    let vault_data: CollateralVault = AnchorDeserialize::deserialize(&mut &vault_account.data[..]).unwrap();
    assert_eq!(vault_data.locked_balance, 500);
    assert_eq!(vault_data.available_balance, 500);
}

// Add more tests similarly for unlock, withdraw, unauthorized, etc.
// For unauthorized lock: Try with invalid caller, expect error
#[tokio::test]
async fn test_unauthorized_lock() {
    // Setup similar to above...
    // Use invalid caller Pubkey
    // Expect process_transaction to return Err with specific error
}

// Run with: cargo test --package collateral_vault --test integration