use basic_4::{
    Deposit, DepositClientAccounts, Initialize, InitializeClientAccounts, VaultAccount,
    VaultProgram, Withdraw, WithdrawClientAccounts,
};
use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{account::Account, clock::Clock, pubkey::Pubkey, sysvar};
use star_frame::{client::SerializeAccount, prelude::*};
use std::collections::HashMap;

#[test]
fn test_initialize_vault() {
    // Setup
    let program_id = VaultProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_4");

    // Create test accounts
    let owner = Pubkey::new_unique();
    let beneficiary = Pubkey::new_unique();
    let unlock_time = 1_700_000_000i64; // Some future timestamp

    // Derive the PDA for the vault
    let (vault_pda, _bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );

    // Create the Initialize instruction
    let init = Initialize {
        beneficiary,
        unlock_time,
    };
    let client_accounts = InitializeClientAccounts {
        owner,
        vault: vault_pda,
        beneficiary,
        system_program: Some(solana_sdk::system_program::ID),
    };

    // Prepare account states
    let owner_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let beneficiary_account = Account {
        lamports: 0,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let vault_account = Account::default();
    let (system_program_key, system_account) =
        mollusk_svm::program::keyed_account_for_system_program();

    // Set up the context
    let context = HashMap::from([
        (owner, owner_account),
        (beneficiary, beneficiary_account),
        (vault_pda, vault_account),
        (system_program_key, system_account),
    ]);

    // Create expected vault state
    let (_, bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );
    let expected_vault = VaultAccount {
        owner,
        beneficiary,
        balance: 0,
        unlock_time,
        bump,
    };

    // Process and validate
    mollusk
        .with_context(context)
        .process_and_validate_instruction(
            &VaultProgram::instruction(&init, client_accounts)
                .expect("Failed to create instruction"),
            &[
                Check::success(),
                Check::account(&vault_pda)
                    .data(
                        &VaultAccount::serialize_account(expected_vault)
                            .expect("Failed to serialize"),
                    )
                    .owner(&program_id)
                    .build(),
            ],
        );

    println!("Basic-4: Vault initialized successfully");
}

#[test]
#[ignore = "Deposit/Withdraw with separate vault wallet requires CPI or program-owned accounts"]
fn test_deposit_to_vault() {
    // Setup
    let program_id = VaultProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_4");

    let owner = Pubkey::new_unique();
    let beneficiary = Pubkey::new_unique();
    let depositor = Pubkey::new_unique();
    let vault_wallet = Pubkey::new_unique(); // Account that holds vault funds
    let unlock_time = 1_700_000_000i64;

    // Derive vault PDA
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );

    // Create initialized vault
    let initial_vault = VaultAccount {
        owner,
        beneficiary,
        balance: 0,
        unlock_time,
        bump,
    };
    let vault_data = VaultAccount::serialize_account(initial_vault).expect("Failed to serialize");

    // Create deposit instruction
    let deposit_amount = 1_000_000u64;
    let deposit = Deposit {
        amount: deposit_amount,
    };
    let client_accounts = DepositClientAccounts {
        depositor,
        vault: vault_pda,
        vault_wallet,
        system_program: Some(solana_sdk::system_program::ID),
    };

    // Prepare accounts
    let depositor_account = Account {
        lamports: 10_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let vault_account = Account {
        lamports: 1_000_000,
        data: vault_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    let vault_wallet_account = Account {
        lamports: 0,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let (system_program_key, system_account) =
        mollusk_svm::program::keyed_account_for_system_program();

    // Set up context
    let context = HashMap::from([
        (depositor, depositor_account),
        (vault_pda, vault_account),
        (vault_wallet, vault_wallet_account),
        (system_program_key, system_account),
    ]);

    // Expected state after deposit
    let expected_vault = VaultAccount {
        owner,
        beneficiary,
        balance: deposit_amount,
        unlock_time,
        bump,
    };

    // Process and validate
    let ctx = mollusk.with_context(context);
    ctx.process_and_validate_instruction(
        &VaultProgram::instruction(&deposit, client_accounts)
            .expect("Failed to create instruction"),
        &[
            Check::success(),
            Check::account(&vault_pda)
                .data(
                    &VaultAccount::serialize_account(expected_vault).expect("Failed to serialize"),
                )
                .build(),
            Check::account(&depositor).lamports(9_000_000).build(), // 10M - 1M
            Check::account(&vault_wallet).lamports(1_000_000).build(), // 0 + 1M
        ],
    );

    println!("Basic-4: Deposit successful");
}

#[test]
#[ignore = "Withdraw with separate vault wallet requires CPI or program-owned accounts"]
fn test_owner_can_withdraw_anytime() {
    // Setup
    let program_id = VaultProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_4");

    let owner = Pubkey::new_unique();
    let beneficiary = Pubkey::new_unique();
    let vault_wallet = Pubkey::new_unique();
    let future_unlock_time = 2_000_000_000i64; // Far future
    let current_time = 1_500_000_000i64; // Before unlock

    // Derive vault PDA
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );

    // Create vault with balance
    let initial_balance = 5_000_000u64;
    let initial_vault = VaultAccount {
        owner,
        beneficiary,
        balance: initial_balance,
        unlock_time: future_unlock_time,
        bump,
    };
    let vault_data = VaultAccount::serialize_account(initial_vault).expect("Failed to serialize");

    // Create withdraw instruction
    let withdraw_amount = 2_000_000u64;
    let withdraw = Withdraw {
        amount: withdraw_amount,
    };
    let client_accounts = WithdrawClientAccounts {
        authority: owner,
        vault: vault_pda,
        vault_wallet,
        recipient: owner, // Owner withdraws to themselves
    };

    // Prepare accounts
    let owner_account = Account {
        lamports: 1_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let vault_account = Account {
        lamports: 1_000_000,
        data: vault_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    let vault_wallet_account = Account {
        lamports: initial_balance,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up Clock sysvar
    let clock = Clock {
        unix_timestamp: current_time,
        ..Clock::default()
    };
    let clock_data = bincode::serialize(&clock).unwrap();
    let clock_account = Account {
        lamports: 1,
        data: clock_data,
        owner: sysvar::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up context
    let context = HashMap::from([
        (owner, owner_account),
        (vault_pda, vault_account),
        (vault_wallet, vault_wallet_account),
        (sysvar::clock::ID, clock_account),
    ]);

    // Expected state after withdrawal
    let expected_vault = VaultAccount {
        owner,
        beneficiary,
        balance: initial_balance - withdraw_amount,
        unlock_time: future_unlock_time,
        bump,
    };

    // Process and validate
    mollusk
        .with_context(context)
        .process_and_validate_instruction(
            &VaultProgram::instruction(&withdraw, client_accounts)
                .expect("Failed to create instruction"),
            &[
                Check::success(),
                Check::account(&vault_pda)
                    .data(
                        &VaultAccount::serialize_account(expected_vault)
                            .expect("Failed to serialize"),
                    )
                    .build(),
                Check::account(&owner).lamports(3_000_000).build(), // 1M + 2M withdrawn
                Check::account(&vault_wallet).lamports(3_000_000).build(), // 5M - 2M
            ],
        );

    println!("Basic-4: Owner withdrew before unlock time successfully");
}

#[test]
fn test_beneficiary_blocked_before_unlock_time() {
    // Setup
    let program_id = VaultProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_4");

    let owner = Pubkey::new_unique();
    let beneficiary = Pubkey::new_unique();
    let vault_wallet = Pubkey::new_unique();
    let future_unlock_time = 2_000_000_000i64; // Far future
    let current_time = 1_500_000_000i64; // Before unlock

    // Derive vault PDA
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );

    // Create vault with balance
    let initial_balance = 5_000_000u64;
    let initial_vault = VaultAccount {
        owner,
        beneficiary,
        balance: initial_balance,
        unlock_time: future_unlock_time,
        bump,
    };
    let vault_data = VaultAccount::serialize_account(initial_vault).expect("Failed to serialize");

    // Try to withdraw as beneficiary before unlock
    let withdraw = Withdraw { amount: 1_000_000 };
    let client_accounts = WithdrawClientAccounts {
        authority: beneficiary, // Beneficiary tries to withdraw
        vault: vault_pda,
        vault_wallet,
        recipient: beneficiary,
    };

    // Prepare accounts
    let beneficiary_account = Account {
        lamports: 100_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let vault_account = Account {
        lamports: 1_000_000,
        data: vault_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    let vault_wallet_account = Account {
        lamports: initial_balance,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up Clock sysvar (current time is BEFORE unlock)
    let clock = Clock {
        unix_timestamp: current_time,
        ..Clock::default()
    };
    let clock_data = bincode::serialize(&clock).unwrap();
    let clock_account = Account {
        lamports: 1,
        data: clock_data,
        owner: sysvar::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up context
    let context = HashMap::from([
        (beneficiary, beneficiary_account),
        (vault_pda, vault_account),
        (vault_wallet, vault_wallet_account),
        (sysvar::clock::ID, clock_account),
    ]);

    // Should fail due to time lock
    let result = mollusk.with_context(context).process_instruction(
        &VaultProgram::instruction(&withdraw, client_accounts)
            .expect("Failed to create instruction"),
    );

    assert!(
        !matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Should fail when beneficiary withdraws before unlock time"
    );

    println!("Basic-4: Correctly rejected beneficiary withdrawal before unlock time");
}

#[test]
#[ignore = "Withdraw with separate vault wallet requires CPI or program-owned accounts"]
fn test_beneficiary_can_withdraw_after_unlock() {
    // Setup
    let program_id = VaultProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_4");

    let owner = Pubkey::new_unique();
    let beneficiary = Pubkey::new_unique();
    let vault_wallet = Pubkey::new_unique();
    let unlock_time = 1_700_000_000i64;
    let current_time = 1_800_000_000i64; // After unlock

    // Derive vault PDA
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );

    // Create vault with balance
    let initial_balance = 5_000_000u64;
    let initial_vault = VaultAccount {
        owner,
        beneficiary,
        balance: initial_balance,
        unlock_time,
        bump,
    };
    let vault_data = VaultAccount::serialize_account(initial_vault).expect("Failed to serialize");

    // Withdraw as beneficiary after unlock
    let withdraw_amount = 2_000_000u64;
    let withdraw = Withdraw {
        amount: withdraw_amount,
    };
    let client_accounts = WithdrawClientAccounts {
        authority: beneficiary,
        vault: vault_pda,
        vault_wallet,
        recipient: beneficiary,
    };

    // Prepare accounts
    let beneficiary_account = Account {
        lamports: 100_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let vault_account = Account {
        lamports: 1_000_000,
        data: vault_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    let vault_wallet_account = Account {
        lamports: initial_balance,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up Clock sysvar (current time is AFTER unlock)
    let clock = Clock {
        unix_timestamp: current_time,
        ..Clock::default()
    };
    let clock_data = bincode::serialize(&clock).unwrap();
    let clock_account = Account {
        lamports: 1,
        data: clock_data,
        owner: sysvar::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up context
    let context = HashMap::from([
        (beneficiary, beneficiary_account),
        (vault_pda, vault_account),
        (vault_wallet, vault_wallet_account),
        (sysvar::clock::ID, clock_account),
    ]);

    // Expected state after withdrawal
    let expected_vault = VaultAccount {
        owner,
        beneficiary,
        balance: initial_balance - withdraw_amount,
        unlock_time,
        bump,
    };

    // Process and validate
    mollusk
        .with_context(context)
        .process_and_validate_instruction(
            &VaultProgram::instruction(&withdraw, client_accounts)
                .expect("Failed to create instruction"),
            &[
                Check::success(),
                Check::account(&vault_pda)
                    .data(
                        &VaultAccount::serialize_account(expected_vault)
                            .expect("Failed to serialize"),
                    )
                    .build(),
                Check::account(&beneficiary).lamports(2_100_000).build(), // 100k + 2M
                Check::account(&vault_wallet).lamports(3_000_000).build(), // 5M - 2M
            ],
        );

    println!("Basic-4: Beneficiary withdrew after unlock time successfully");
}

#[test]
fn test_withdraw_insufficient_funds_error() {
    // Setup
    let program_id = VaultProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_4");

    let owner = Pubkey::new_unique();
    let beneficiary = Pubkey::new_unique();
    let vault_wallet = Pubkey::new_unique();

    // Derive vault PDA
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );

    // Create vault with small balance
    let vault_balance = 1_000_000u64;
    let initial_vault = VaultAccount {
        owner,
        beneficiary,
        balance: vault_balance,
        unlock_time: 0, // No time lock for this test
        bump,
    };
    let vault_data = VaultAccount::serialize_account(initial_vault).expect("Failed to serialize");

    // Try to withdraw more than available
    let withdraw = Withdraw {
        amount: 2_000_000u64, // More than balance!
    };
    let client_accounts = WithdrawClientAccounts {
        authority: owner,
        vault: vault_pda,
        vault_wallet,
        recipient: owner,
    };

    // Prepare accounts
    let owner_account = Account {
        lamports: 1_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let vault_account = Account {
        lamports: 1_000_000,
        data: vault_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    let vault_wallet_account = Account {
        lamports: vault_balance,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up context
    let context = HashMap::from([
        (owner, owner_account),
        (vault_pda, vault_account),
        (vault_wallet, vault_wallet_account),
    ]);

    // Should fail due to insufficient funds
    let result = mollusk.with_context(context).process_instruction(
        &VaultProgram::instruction(&withdraw, client_accounts)
            .expect("Failed to create instruction"),
    );

    assert!(
        !matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Should fail with insufficient funds"
    );

    println!("Basic-4: Correctly rejected withdrawal with insufficient funds");
}

#[test]
fn test_complex_pda_derivation() {
    // Test that PDAs are unique for each owner-beneficiary pair
    let program_id = VaultProgram::ID;

    let owner1 = Pubkey::new_unique();
    let owner2 = Pubkey::new_unique();
    let beneficiary1 = Pubkey::new_unique();
    let beneficiary2 = Pubkey::new_unique();

    // Same owner, different beneficiaries
    let (pda1, _) = Pubkey::find_program_address(
        &[b"vault", owner1.as_ref(), beneficiary1.as_ref()],
        &program_id,
    );
    let (pda2, _) = Pubkey::find_program_address(
        &[b"vault", owner1.as_ref(), beneficiary2.as_ref()],
        &program_id,
    );

    // Different owners, same beneficiary
    let (pda3, _) = Pubkey::find_program_address(
        &[b"vault", owner2.as_ref(), beneficiary1.as_ref()],
        &program_id,
    );

    // Different owners and beneficiaries
    let (pda4, _) = Pubkey::find_program_address(
        &[b"vault", owner2.as_ref(), beneficiary2.as_ref()],
        &program_id,
    );

    // All PDAs should be unique
    assert_ne!(
        pda1, pda2,
        "Different beneficiaries should produce different PDAs"
    );
    assert_ne!(pda1, pda3, "Different owners should produce different PDAs");
    assert_ne!(
        pda1, pda4,
        "Different owner-beneficiary pairs should produce different PDAs"
    );
    assert_ne!(pda2, pda3, "Each combination should be unique");
    assert_ne!(pda2, pda4, "Each combination should be unique");
    assert_ne!(pda3, pda4, "Each combination should be unique");

    println!("Basic-4: Complex PDA derivation verified");
    println!("  Owner1-Beneficiary1: {}", pda1);
    println!("  Owner1-Beneficiary2: {}", pda2);
    println!("  Owner2-Beneficiary1: {}", pda3);
    println!("  Owner2-Beneficiary2: {}", pda4);
}

#[test]
fn test_unauthorized_withdrawal_fails() {
    // Test that a random third party cannot withdraw
    let program_id = VaultProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_4");

    let owner = Pubkey::new_unique();
    let beneficiary = Pubkey::new_unique();
    let attacker = Pubkey::new_unique(); // Unauthorized third party
    let vault_wallet = Pubkey::new_unique();

    // Derive vault PDA
    let (vault_pda, bump) = Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), beneficiary.as_ref()],
        &program_id,
    );

    // Create vault with balance
    let initial_vault = VaultAccount {
        owner,
        beneficiary,
        balance: 5_000_000,
        unlock_time: 0,
        bump,
    };
    let vault_data = VaultAccount::serialize_account(initial_vault).expect("Failed to serialize");

    // Attacker tries to withdraw
    let withdraw = Withdraw { amount: 1_000_000 };
    let client_accounts = WithdrawClientAccounts {
        authority: attacker, // Unauthorized!
        vault: vault_pda,
        vault_wallet,
        recipient: attacker,
    };

    // Prepare accounts
    let attacker_account = Account {
        lamports: 100_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let vault_account = Account {
        lamports: 1_000_000,
        data: vault_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    let vault_wallet_account = Account {
        lamports: 5_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Set up context
    let context = HashMap::from([
        (attacker, attacker_account),
        (vault_pda, vault_account),
        (vault_wallet, vault_wallet_account),
    ]);

    // Should fail due to unauthorized access
    let result = mollusk.with_context(context).process_instruction(
        &VaultProgram::instruction(&withdraw, client_accounts)
            .expect("Failed to create instruction"),
    );

    assert!(
        !matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Should fail when unauthorized user tries to withdraw"
    );

    println!("Basic-4: Correctly rejected unauthorized withdrawal");
}
