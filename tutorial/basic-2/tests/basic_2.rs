use basic_2::{
    CounterAccount, CounterProgram, Increment, IncrementClientAccounts, Initialize,
    InitializeClientAccounts,
};
use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use star_frame::{client::SerializeAccount, prelude::*};
use std::collections::HashMap;

#[test]
fn test_initialize_counter() {
    // Setup
    let program_id = CounterProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_2");

    // Create test accounts
    let authority = Pubkey::new_unique();

    // Derive the PDA for the counter account
    let (counter_pda, _bump) =
        Pubkey::find_program_address(&[b"counter", authority.as_ref()], &program_id);

    // Create the Initialize instruction
    let init = Initialize;
    let client_accounts = InitializeClientAccounts {
        authority,
        counter: counter_pda,
        system_program: Some(solana_sdk::system_program::ID),
    };

    // Prepare initial account states
    let authority_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let counter_account = Account::default();
    let (system_program_key, system_account) =
        mollusk_svm::program::keyed_account_for_system_program();

    // Set up the context
    let context = HashMap::from([
        (authority, authority_account),
        (counter_pda, counter_account),
        (system_program_key, system_account),
    ]);

    // Create expected counter state after initialization
    let expected_counter = CounterAccount {
        authority,
        count: 0,
    };

    // Process and validate the instruction
    mollusk
        .with_context(context)
        .process_and_validate_instruction(
            &CounterProgram::instruction(&init, client_accounts)
                .expect("Failed to create instruction"),
            &[
                Check::success(),
                Check::account(&counter_pda)
                    .data(
                        &CounterAccount::serialize_account(expected_counter)
                            .expect("Failed to serialize"),
                    )
                    .owner(&program_id)
                    .build(),
            ],
        );

    println!("Basic-2: Counter initialized successfully");
}

#[test]
fn test_increment_with_valid_authority() {
    // Setup
    let program_id = CounterProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_2");

    // Create test accounts
    let authority = Pubkey::new_unique();

    // Derive the PDA for the counter account
    let (counter_pda, _bump) =
        Pubkey::find_program_address(&[b"counter", authority.as_ref()], &program_id);

    // Create an already initialized counter
    let initial_counter = CounterAccount {
        authority,
        count: 5,
    };
    let counter_data =
        CounterAccount::serialize_account(initial_counter).expect("Failed to serialize");

    // Create the Increment instruction
    let increment = Increment;
    let client_accounts = IncrementClientAccounts {
        authority,
        counter: counter_pda,
    };

    // Prepare accounts
    let authority_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let counter_account = Account {
        lamports: 1_000_000,
        data: counter_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    // Set up the context
    let context = HashMap::from([
        (authority, authority_account),
        (counter_pda, counter_account),
    ]);

    // Create expected counter state after increment
    let expected_counter = CounterAccount {
        authority,
        count: 6, // Should be incremented by 1
    };

    // Process and validate the instruction
    mollusk
        .with_context(context)
        .process_and_validate_instruction(
            &CounterProgram::instruction(&increment, client_accounts)
                .expect("Failed to create instruction"),
            &[
                Check::success(),
                Check::account(&counter_pda)
                    .data(
                        &CounterAccount::serialize_account(expected_counter)
                            .expect("Failed to serialize"),
                    )
                    .owner(&program_id)
                    .build(),
            ],
        );

    println!("Basic-2: Counter incremented successfully by valid authority");
}

#[test]
fn test_increment_with_invalid_authority_fails() {
    // Setup
    let program_id = CounterProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_2");

    // Create test accounts
    let real_authority = Pubkey::new_unique();
    let fake_authority = Pubkey::new_unique(); // Different from real_authority

    // Derive the PDA for the counter account (using real authority)
    let (counter_pda, _bump) =
        Pubkey::find_program_address(&[b"counter", real_authority.as_ref()], &program_id);

    // Create an already initialized counter with real_authority
    let initial_counter = CounterAccount {
        authority: real_authority,
        count: 5,
    };
    let counter_data =
        CounterAccount::serialize_account(initial_counter).expect("Failed to serialize");

    // Try to increment with fake_authority
    let increment = Increment;
    let client_accounts = IncrementClientAccounts {
        authority: fake_authority, // Using wrong authority!
        counter: counter_pda,
    };

    // Prepare accounts
    let fake_authority_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let counter_account = Account {
        lamports: 1_000_000,
        data: counter_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    // Set up the context
    let context = HashMap::from([
        (fake_authority, fake_authority_account),
        (counter_pda, counter_account),
    ]);

    // Process the instruction - should fail validation
    let result = mollusk.with_context(context).process_instruction(
        &CounterProgram::instruction(&increment, client_accounts)
            .expect("Failed to create instruction"),
    );

    // Verify it failed
    use mollusk_svm::result::ProgramResult;
    assert!(
        !matches!(result.program_result, ProgramResult::Success),
        "Should fail with invalid authority"
    );

    println!("Basic-2: Correctly rejected increment from invalid authority");
}

#[test]
fn test_multiple_increments() {
    // Setup
    let program_id = CounterProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_2");

    // Create test accounts
    let authority = Pubkey::new_unique();

    // Derive the PDA for the counter account
    let (counter_pda, _bump) =
        Pubkey::find_program_address(&[b"counter", authority.as_ref()], &program_id);

    // Initialize the counter first
    let init = Initialize;
    let init_accounts = InitializeClientAccounts {
        authority,
        counter: counter_pda,
        system_program: Some(solana_sdk::system_program::ID),
    };

    let authority_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let counter_account = Account::default();
    let (system_program_key, system_account) =
        mollusk_svm::program::keyed_account_for_system_program();

    let mut context = HashMap::from([
        (authority, authority_account.clone()),
        (counter_pda, counter_account),
        (system_program_key, system_account),
    ]);

    // Initialize
    let mollusk_context = mollusk.with_context(context.clone());
    mollusk_context.process_and_validate_instruction(
        &CounterProgram::instruction(&init, init_accounts).expect("Failed to create instruction"),
        &[Check::success()],
    );

    // Get the initialized account from the account store
    let initialized_account = mollusk_context
        .account_store
        .borrow()
        .get(&counter_pda)
        .cloned()
        .expect("Counter should exist");

    // Update context with initialized account
    context.insert(counter_pda, initialized_account);
    context.remove(&system_program_key); // No longer needed

    // Perform multiple increments
    let increment = Increment;

    let expected_counts = [1, 2, 3, 4, 5];

    for expected_count in expected_counts {
        // Create increment accounts each time (since it gets moved)
        let increment_accounts = IncrementClientAccounts {
            authority,
            counter: counter_pda,
        };

        // Get current account state
        let _current_account = context.get(&counter_pda).unwrap().clone();

        // Process increment
        let mollusk_ctx =
            Mollusk::new(&program_id, "target/deploy/basic_2").with_context(context.clone());
        let result = mollusk_ctx.process_instruction(
            &CounterProgram::instruction(&increment, increment_accounts)
                .expect("Failed to create instruction"),
        );

        assert!(matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ));

        // Get updated account from the result
        let updated_account = mollusk_ctx
            .account_store
            .borrow()
            .get(&counter_pda)
            .cloned()
            .expect("Counter account should exist");

        // Update context with new state
        context.insert(counter_pda, updated_account.clone());

        // Verify count incremented correctly
        let counter_data = &updated_account.data;
        // Skip discriminant (8 bytes), authority (32 bytes), then read count (8 bytes)
        let count_bytes = &counter_data[40..48];
        let actual_count = u64::from_le_bytes(count_bytes.try_into().unwrap());

        assert_eq!(
            actual_count, expected_count,
            "Count should be {} after increment #{}",
            expected_count, expected_count
        );
    }

    println!("Basic-2: Successfully performed 5 increments");
}

#[test]
fn test_pda_derivation_with_different_authorities() {
    // Test that different authorities produce different PDAs
    let program_id = CounterProgram::ID;

    let authority1 = Pubkey::new_unique();
    let authority2 = Pubkey::new_unique();

    let (pda1, bump1) =
        Pubkey::find_program_address(&[b"counter", authority1.as_ref()], &program_id);
    let (pda2, bump2) =
        Pubkey::find_program_address(&[b"counter", authority2.as_ref()], &program_id);

    // Different authorities should produce different PDAs
    assert_ne!(
        pda1, pda2,
        "Different authorities should have different PDAs"
    );

    // Both should be valid PDAs (not on the curve)
    assert!(!pda1.is_on_curve());
    assert!(!pda2.is_on_curve());

    println!("Basic-2: PDA derivation works correctly for different authorities");
    println!("  Authority 1 PDA: {} (bump: {})", pda1, bump1);
    println!("  Authority 2 PDA: {} (bump: {})", pda2, bump2);
}

#[test]
fn test_counter_state_persistence() {
    // Test that counter state persists correctly across operations
    let program_id = CounterProgram::ID;
    let _mollusk = Mollusk::new(&program_id, "target/deploy/basic_2");

    let authority = Pubkey::new_unique();
    let (_counter_pda, _) =
        Pubkey::find_program_address(&[b"counter", authority.as_ref()], &program_id);

    // Create counter with specific initial state
    let test_count = 42u64;
    let counter_state = CounterAccount {
        authority,
        count: test_count,
    };

    let serialized = CounterAccount::serialize_account(counter_state).expect("Failed to serialize");

    // Verify serialization preserves state
    let counter_account = Account {
        lamports: 1_000_000,
        data: serialized.clone(),
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    // Read back the count from serialized data
    // Skip discriminant (8 bytes), authority (32 bytes), then read count (8 bytes)
    let count_bytes = &counter_account.data[40..48];
    let deserialized_count = u64::from_le_bytes(count_bytes.try_into().unwrap());

    assert_eq!(
        deserialized_count, test_count,
        "State should persist correctly"
    );

    println!("Basic-2: Counter state persistence verified");
}

#[test]
fn test_unsigned_increment_fails() {
    // Test that increment fails without a signature
    let program_id = CounterProgram::ID;
    let authority = Pubkey::new_unique();
    let (counter_pda, _) =
        Pubkey::find_program_address(&[b"counter", authority.as_ref()], &program_id);

    // Create an already initialized counter
    let initial_counter = CounterAccount {
        authority,
        count: 5,
    };
    let counter_data =
        CounterAccount::serialize_account(initial_counter).expect("Failed to serialize");

    // Create increment instruction but mark authority as non-signer
    let increment = Increment;

    // Manually create instruction with authority as non-signer
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(authority, false), // false = not a signer!
            AccountMeta::new(counter_pda, false),
        ],
        data: CounterProgram::instruction(
            &increment,
            IncrementClientAccounts {
                authority,
                counter: counter_pda,
            },
        )
        .expect("Failed to create instruction")
        .data,
    };

    // Try to process without signature
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_2");

    let authority_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    let counter_account = Account {
        lamports: 1_000_000,
        data: counter_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    let context = HashMap::from([
        (authority, authority_account),
        (counter_pda, counter_account),
    ]);

    // This should fail because authority is not marked as signer
    let result = mollusk
        .with_context(context)
        .process_instruction(&instruction);

    assert!(
        !matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Should fail without signature"
    );

    println!("Basic-2: Correctly rejected unsigned increment");
}
