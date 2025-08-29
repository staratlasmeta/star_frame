use basic_1::{
    BasicProgram, DataAccount, Initialize, InitializeClientAccounts, Update, UpdateClientAccounts,
};
use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{account::Account, pubkey::Pubkey};
use star_frame::{client::SerializeAccount, prelude::*};
use std::collections::HashMap;

#[test]
fn test_initialize_instruction() {
    // Setup
    let program_id = BasicProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_1");

    // Create test accounts
    let authority = Pubkey::new_unique();

    // Derive the PDA for the data account
    let (data_account_pda, _bump) =
        Pubkey::find_program_address(&[b"data", authority.as_ref()], &program_id);

    // Create the Initialize instruction using the generated client code
    let initial_value = 42u64;
    let init = Initialize { initial_value };

    // Use the generated ClientAccounts type
    let client_accounts = InitializeClientAccounts {
        authority,
        data_account: data_account_pda,
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

    // Data account starts as a system account (uninitialized)
    let data_account = Account::default();

    let (system_program_key, system_account) =
        mollusk_svm::program::keyed_account_for_system_program();

    // Set up the context as a HashMap
    let context = HashMap::from([
        (authority, authority_account),
        (data_account_pda, data_account),
        (system_program_key, system_account),
    ]);

    // Create expected data account after initialization
    let expected_data_account = DataAccount {
        data: initial_value,
    };

    // Process and validate the instruction
    mollusk
        .with_context(context)
        .process_and_validate_instruction(
            &BasicProgram::instruction(&init, client_accounts)
                .expect("Failed to create instruction"),
            &[
                Check::success(),
                Check::account(&data_account_pda)
                    .data(
                        &DataAccount::serialize_account(expected_data_account)
                            .expect("Failed to serialize"),
                    )
                    .owner(&program_id)
                    .build(),
            ],
        );

    println!("Basic-1: Initialize instruction executed successfully");
}

#[test]
fn test_update_instruction() {
    // Setup
    let program_id = BasicProgram::ID;
    let mollusk = Mollusk::new(&program_id, "target/deploy/basic_1");

    // Create test accounts
    let authority = Pubkey::new_unique();

    // Derive the PDA for the data account
    let (data_account_pda, _bump) =
        Pubkey::find_program_address(&[b"data", authority.as_ref()], &program_id);

    // Create an already initialized data account with proper serialization
    let initial_data = DataAccount { data: 100 };
    let account_data =
        DataAccount::serialize_account(initial_data).expect("Failed to serialize initial data");

    // Create the Update instruction using the generated client code
    let new_value = 200u64;
    let update = Update { value: new_value };

    // Use the generated ClientAccounts type
    let client_accounts = UpdateClientAccounts {
        authority,
        data_account: data_account_pda,
    };

    // Prepare accounts for the test
    let authority_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: solana_sdk::system_program::ID,
        executable: false,
        rent_epoch: 0,
    };

    // Data account is already initialized and owned by the program
    let data_account = Account {
        lamports: 1_000_000,
        data: account_data,
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    };

    // Set up the context as a HashMap
    let context = HashMap::from([
        (authority, authority_account),
        (data_account_pda, data_account),
    ]);

    // Create expected data account after update
    let expected_data_account = DataAccount { data: new_value };

    // Process and validate the instruction
    mollusk
        .with_context(context)
        .process_and_validate_instruction(
            &BasicProgram::instruction(&update, client_accounts)
                .expect("Failed to create instruction"),
            &[
                Check::success(),
                Check::account(&data_account_pda)
                    .data(
                        &DataAccount::serialize_account(expected_data_account)
                            .expect("Failed to serialize"),
                    )
                    .owner(&program_id)
                    .build(),
            ],
        );

    println!("Basic-1: Update instruction executed successfully");
}

#[test]
fn test_pda_derivation() {
    // Test that PDAs are derived correctly
    let program_id = BasicProgram::ID;
    let authority = Pubkey::new_unique();

    // Derive PDA using the same seeds as the program
    let (pda, bump) = Pubkey::find_program_address(&[b"data", authority.as_ref()], &program_id);

    // Verify the PDA is not on the curve (which is what makes it a valid PDA)
    assert!(!pda.is_on_curve());

    // Verify we can recreate the same PDA with the bump
    let recreated_pda =
        Pubkey::create_program_address(&[b"data", authority.as_ref(), &[bump]], &program_id)
            .expect("Should create valid PDA");

    assert_eq!(pda, recreated_pda);
    println!("Basic-1: PDA derivation verified");
}

#[test]
fn test_data_account_serialization() {
    // Test that DataAccount can be properly serialized using star_frame's SerializeAccount
    let test_value = 12345u64;
    let data_account = DataAccount { data: test_value };

    // Serialize using star_frame's client serialization
    let serialized =
        DataAccount::serialize_account(data_account).expect("Failed to serialize account");

    // Verify the serialized data contains our value
    // The first 8 bytes should be a discriminant, followed by the data
    assert!(
        serialized.len() >= 16,
        "Serialized data should include discriminant and data"
    );

    // Extract the data portion (skip the 8-byte discriminant)
    let data_bytes = &serialized[8..16];
    let deserialized_value = u64::from_le_bytes(data_bytes.try_into().unwrap());
    assert_eq!(deserialized_value, test_value);

    println!("Basic-1: DataAccount serialization test passed");
}
