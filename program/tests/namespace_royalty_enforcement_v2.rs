#![cfg(feature = "test-bpf")]

pub mod utils;

use mpl_token_auth_rules::{
    error::RuleSetError,
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{All, Amount, IsWallet, Namespace, Operator, ProgramOwnedList, RuleSetV2},
};
use solana_program::{instruction::InstructionError, pubkey, pubkey::Pubkey};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{
    instruction::AccountMeta,
    signature::Signer,
    signer::keypair::Keypair,
    system_instruction,
    transaction::{Transaction, TransactionError},
};
use utils::{
    create_associated_token_account, create_mint, program_test, DelegateScenario,
    MetadataDelegateRole, Operation, PayloadKey, TokenDelegateRole, TransferScenario,
};

const ADDITIONAL_COMPUTE: u32 = 400_000;
const RULE_SET_NAME: &str = "Metaplex Royalty RuleSet Dev";

// --------------------------------
// Define Program Allow List
// --------------------------------
const ROOSTER_PROGRAM_ID: Pubkey = pubkey!("roos9SDjRQhy5iq8gVihwBoVvgYcNDxqhj1HdhGpiu5");
const TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("Meta88XpDHcSJZDFiHop6c9sXaufkZX5depkZyrYBWv");
const TOKEN_AUTH_RULES_ID: Pubkey = pubkey!("AuthxYNhPnnrGBo1wdzeUdukrsFpHvR42wghx8ZPNEo4");

const TRANSFER_PROGRAM_BASE_ALLOW_LIST: [Pubkey; 3] = [
    TOKEN_METADATA_PROGRAM_ID,
    ROOSTER_PROGRAM_ID,
    TOKEN_AUTH_RULES_ID,
];
const DELEGATE_PROGRAM_BASE_ALLOW_LIST: [Pubkey; 3] = [
    TOKEN_METADATA_PROGRAM_ID,
    ROOSTER_PROGRAM_ID,
    TOKEN_AUTH_RULES_ID,
];
const ADVANCED_DELEGATE_PROGRAM_BASE_ALLOW_LIST: [Pubkey; 3] = [
    TOKEN_METADATA_PROGRAM_ID,
    ROOSTER_PROGRAM_ID,
    TOKEN_AUTH_RULES_ID,
];

struct ComposedRules {
    transfer_rule: Vec<u8>,
    wallet_to_wallet_rule: Vec<u8>,
    delegate_rule: Vec<u8>,
    advanced_delegate_rule: Vec<u8>,
    namespace_rule: Vec<u8>,
}

// Get the four Composed Rules used in this RuleSet.
fn get_composed_rules() -> ComposedRules {
    // --------------------------------
    // Create Primitive Rules
    // --------------------------------
    let nft_amount = Amount::serialize(PayloadKey::Amount.to_string(), Operator::Eq, 1).unwrap();

    // Generate some random programs to add to the base lists.
    let random_programs = (0..18).map(|_| Keypair::new().pubkey()).collect::<Vec<_>>();

    // Create a Rule.  The target must be owned by the program ID specified in the Rule.
    let multi_field_program_allow_list = ProgramOwnedList::serialize(
        format!(
            "{}|{}|{}",
            PayloadKey::Source.to_string(),
            PayloadKey::Destination.to_string(),
            PayloadKey::Authority.to_string()
        ),
        &[
            TRANSFER_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
    )
    .unwrap();

    let source_is_wallet = IsWallet::serialize(PayloadKey::Source.to_string()).unwrap();

    let dest_is_wallet = IsWallet::serialize(PayloadKey::Destination.to_string()).unwrap();

    let delegate_program_allow_list = ProgramOwnedList::serialize(
        PayloadKey::Delegate.to_string(),
        &[
            DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
    )
    .unwrap();

    let advanced_delegate_program_allow_list = ProgramOwnedList::serialize(
        PayloadKey::Delegate.to_string(),
        &[
            ADVANCED_DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs,
        ]
        .concat(),
    )
    .unwrap();

    // --------------------------------
    // Create Composed Rules from
    // Primitive Rules
    // --------------------------------
    // amount is 1 && (source owner on allow list || dest owner on allow list || authority owner on allow list )
    let transfer_rule = All::serialize(&[&nft_amount, &multi_field_program_allow_list]).unwrap();

    // (amount is 1 && source is wallet && dest is wallet)
    let wallet_to_wallet_rule =
        All::serialize(&[&nft_amount, &source_is_wallet, &dest_is_wallet]).unwrap();

    let delegate_rule = All::serialize(&[&nft_amount, &delegate_program_allow_list]).unwrap();

    let advanced_delegate_rule =
        All::serialize(&[&nft_amount, &advanced_delegate_program_allow_list]).unwrap();

    let namespace_rule = Namespace::serialize().unwrap();

    ComposedRules {
        transfer_rule,
        wallet_to_wallet_rule,
        delegate_rule,
        advanced_delegate_rule,
        namespace_rule,
    }
}

fn get_royalty_rule_set(owner: Pubkey, omit: &str) -> Vec<u8> {
    // Get transfer and wallet-to-wallet rules.
    let royalty_rules = get_composed_rules();

    let mut operations = Vec::new();
    let mut rules: Vec<&[u8]> = Vec::new();

    // --------------------------------
    // Set up transfer operations
    // --------------------------------

    let operation = Operation::TransferNamespace.to_string();
    if omit != operation {
        operations.push(operation);
        rules.push(&royalty_rules.transfer_rule);
    }

    let operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    }
    .to_string();
    if omit != operation {
        operations.push(operation);
        rules.push(&royalty_rules.namespace_rule);
    }

    let operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    }
    .to_string();
    if omit != operation {
        operations.push(operation);
        rules.push(&royalty_rules.namespace_rule);
    }

    let operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    }
    .to_string();
    if omit != operation {
        operations.push(operation);
        rules.push(&royalty_rules.namespace_rule);
    }

    let operation = Operation::Transfer {
        scenario: TransferScenario::MigrationDelegate,
    }
    .to_string();
    if omit != operation {
        operations.push(operation);
        rules.push(&royalty_rules.namespace_rule);
    }

    let operation = Operation::Transfer {
        scenario: TransferScenario::WalletToWallet,
    }
    .to_string();
    if omit != operation {
        operations.push(operation);
        rules.push(&royalty_rules.wallet_to_wallet_rule);
    }

    // --------------------------------
    // Setup metadata delegate operations
    // --------------------------------

    operations.push(Operation::DelegateNamespace.to_string());
    rules.push(&royalty_rules.delegate_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Metadata(MetadataDelegateRole::Authority),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Metadata(MetadataDelegateRole::Collection),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Metadata(MetadataDelegateRole::Use),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Metadata(MetadataDelegateRole::Update),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    // --------------------------------
    // Setup token delegate operations
    // --------------------------------

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Token(TokenDelegateRole::Sale),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Token(TokenDelegateRole::Transfer),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Token(TokenDelegateRole::LockedTransfer),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.advanced_delegate_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Token(TokenDelegateRole::Utility),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    operations.push(
        Operation::Delegate {
            scenario: DelegateScenario::Token(TokenDelegateRole::Staking),
        }
        .to_string(),
    );
    rules.push(&royalty_rules.namespace_rule);

    // Create a RuleSetV2.
    RuleSetV2::serialize(owner, RULE_SET_NAME, &operations, &rules).unwrap()
}

async fn create_royalty_rule_set(context: &mut ProgramTestContext) -> Pubkey {
    let royalty_rule_set = get_royalty_rule_set(context.payer.pubkey(), "");

    // Put the `RuleSet` on chain.
    create_big_rule_set_on_chain!(
        context,
        royalty_rule_set,
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await
}

async fn create_incomplete_royalty_rule_set(
    context: &mut ProgramTestContext,
    missing_op: String,
) -> Pubkey {
    let royalty_rule_set = get_royalty_rule_set(context.payer.pubkey(), &missing_op);

    // Put the `RuleSet` on chain.
    create_big_rule_set_on_chain!(
        context,
        royalty_rule_set,
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await
}

#[tokio::test]
async fn create_rule_set() {
    let mut context = program_test().start_with_context().await;
    let _rule_set_addr = create_royalty_rule_set(&mut context).await;
}

#[tokio::test]
async fn wallet_to_wallet_unimplemented() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Create source and destination wallets.
    let source = Keypair::new();
    let dest = Keypair::new();

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
    ]);

    let transfer_wallet_to_wallet_operation = Operation::Transfer {
        scenario: TransferScenario::WalletToWallet,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(dest.pubkey(), false),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_wallet_to_wallet_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate fail operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  The `IsWallet` rule currently returns `NotImplemented`.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::NotImplemented as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn wallet_to_prog_owned() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Our destination key is going to be an account owned by the mpl-token-auth-rules program.
    // Any one will do so for convenience we just use the RuleSet.

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE)).await;
}

#[tokio::test]
async fn wallet_to_prog_owned_missing_namespace() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr =
        create_incomplete_royalty_rule_set(&mut context, "Transfer:Owner".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Our destination key is going to be an account owned by the mpl-token-auth-rules program.
    // Any one will do so for convenience we just use the RuleSet.

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Program owner was not on the allow list.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::OperationNotFound as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn wallet_to_prog_owned_no_default() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr =
        create_incomplete_royalty_rule_set(&mut context, "Transfer".to_string()).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Our destination key is going to be an account owned by the mpl-token-auth-rules program.
    // Any one will do so for convenience we just use the RuleSet.

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Program owner was not on the allow list.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::OperationNotFound as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn prog_owned_to_prog_owned() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our source and destination keys are going to be accounts owned by the mpl-token-auth-rules
    // program.  Any one will do so for convenience we just use two `RuleSets`.

    // Get first on-chain account.
    let first_on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(first_on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, first_on_chain_account.owner);

    // Create destination `RuleSet`.
    let second_rule_set =
        RuleSetV2::serialize(context.payer.pubkey(), "second_rule_set", &[], &[]).unwrap();

    let second_rule_set_addr = create_rule_set_on_chain_serialized!(
        &mut context,
        second_rule_set,
        "second_rule_set".to_string()
    )
    .await;

    // Get second on-chain account.
    let second_on_chain_account = context
        .banks_client
        .get_account(second_rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(second_on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, second_on_chain_account.owner);

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(second_rule_set_addr),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(second_rule_set_addr, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_transfer_delegate_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE)).await;
}

#[tokio::test]
async fn prog_owned_to_wallet() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our source key is going to be an account owned by the mpl-token-auth-rules program.  Any one
    // will do so for convenience we just use the `RuleSet`.

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    // Destination key is a wallet.
    let dest = Keypair::new();

    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_sale_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(dest.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_sale_delegate_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE)).await;
}

#[tokio::test]
async fn wrong_amount_fails() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Our source key is going to be an account owned by the mpl-token-auth-rules program.  Any one
    // will do so for convenience we just use the `RuleSet`.

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(rule_set_addr)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    // Destination key is a wallet.
    let dest = Keypair::new();

    // Store the payload of data to validate against the rule definition, using the WRONG amount.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(2)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(rule_set_addr),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(dest.pubkey()),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_sale_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::SaleDelegate,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(rule_set_addr, false),
            AccountMeta::new_readonly(dest.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_sale_delegate_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Amount was greater than that allowed in the rule so it
    // failed.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::AmountCheckFailed as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn prog_owner_not_on_list_fails() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Create an associated token account for the sole purpose of having an account that is owned
    // by a different program than what is in the rule.
    create_mint(
        &mut context,
        &mint,
        &source.pubkey(),
        Some(&source.pubkey()),
        0,
    )
    .await
    .unwrap();

    let associated_token_account =
        create_associated_token_account(&mut context, &source, &mint.pubkey())
            .await
            .unwrap();

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(associated_token_account)
        .await
        .unwrap()
        .unwrap();

    // Account must have nonzero data to count as program-owned.
    assert!(on_chain_account.data.iter().any(|&x| x != 0));

    // Verify account ownership.
    assert_eq!(spl_token::ID, on_chain_account.owner);

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(associated_token_account),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(associated_token_account, false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Program owner was not on the allow list.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::ProgramOwnedListCheckFailed as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}

#[tokio::test]
async fn prog_owned_but_zero_data_length() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set(&mut context).await;

    // Create a Keypair to simulate a token mint address.
    let mint = Keypair::new();

    // Source key is a wallet.
    let source = Keypair::new();

    // Create an account owned by mpl-token-auth-rules.
    let program_owned_account = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &context.payer.pubkey(),
            &program_owned_account.pubkey(),
            rent.minimum_balance(0),
            0,
            &mpl_token_auth_rules::ID,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &program_owned_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    // Get on-chain account.
    let on_chain_account = context
        .banks_client
        .get_account(program_owned_account.pubkey())
        .await
        .unwrap()
        .unwrap();

    // Verify data length is zero.
    assert_eq!(0, on_chain_account.data.len());

    // Verify account ownership.
    assert_eq!(mpl_token_auth_rules::ID, on_chain_account.owner);

    // Store the payload of data to validate against the rule definition.
    let payload = Payload::from([
        (PayloadKey::Amount.to_string(), PayloadType::Number(1)),
        (
            PayloadKey::Source.to_string(),
            PayloadType::Pubkey(source.pubkey()),
        ),
        (
            PayloadKey::Destination.to_string(),
            PayloadType::Pubkey(program_owned_account.pubkey()),
        ),
        (
            PayloadKey::Authority.to_string(),
            PayloadType::Pubkey(context.payer.pubkey()),
        ),
    ]);

    let transfer_owner_operation = Operation::Transfer {
        scenario: TransferScenario::Holder,
    };

    // Create a `validate` instruction.
    let validate_ix = ValidateBuilder::new()
        .rule_set_pda(rule_set_addr)
        .mint(mint.pubkey())
        .additional_rule_accounts(vec![
            AccountMeta::new_readonly(source.pubkey(), false),
            AccountMeta::new_readonly(program_owned_account.pubkey(), false),
            AccountMeta::new_readonly(context.payer.pubkey(), true),
        ])
        .build(ValidateArgs::V1 {
            operation: transfer_owner_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Fail to validate operation.
    let err =
        process_failing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE))
            .await;

    // Check that error is what we expect.  Although the program owner is correct the data length is zero
    // so it fails the rule.
    match err {
        solana_program_test::BanksClientError::TransactionError(
            TransactionError::InstructionError(_, InstructionError::Custom(error)),
        ) => {
            assert_eq!(error, RuleSetError::DataIsEmpty as u32);
        }
        _ => panic!("Unexpected error: {:?}", err),
    }
}
