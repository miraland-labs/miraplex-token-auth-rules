#![cfg(feature = "test-bpf")]

pub mod utils;

use arrayref::array_ref;
use mpl_token_auth_rules::{
    bytemuck::{All, Amount, ProgramOwnedList, RuleSetV2},
    instruction::{builders::ValidateBuilder, InstructionBuilder, ValidateArgs},
    payload::{Payload, PayloadType},
    state::{CompareOp, Key, Rule, RuleSetV1},
};
use solana_program::{pubkey, pubkey::Pubkey};
use solana_program_test::{tokio, ProgramTestContext};
use solana_sdk::{
    commitment_config::CommitmentLevel, instruction::AccountMeta, signature::Signer,
    signer::keypair::Keypair,
};
use utils::{
    program_test, DelegateScenario, MetadataDelegateRole, Operation, PayloadKey, TokenDelegateRole,
    TransferScenario,
};

const ADDITIONAL_COMPUTE: u32 = 400_000;
const RULE_SET_NAME: &str = "Metaplex Royalty RuleSet Dev";

// --------------------------------
// Define Program Allow List
// --------------------------------
const ROOSTER_PROGRAM_ID: Pubkey = pubkey!("Roostrnex2Z9Y2XZC49sFAdZARP8E4iFpEnZC5QJWdz");
const TOKEN_METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
const TOKEN_AUTH_RULES_ID: Pubkey = pubkey!("auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg");

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

struct ComposedRulesV1 {
    transfer_rule: Rule,
    delegate_rule: Rule,
    advanced_delegate_rule: Rule,
}

struct ComposedRulesV2 {
    transfer_rule: Vec<u8>,
    delegate_rule: Vec<u8>,
    advanced_delegate_rule: Vec<u8>,
}

// Get the four Composed Rules used in this RuleSet.
fn get_composed_rules_v2() -> ComposedRulesV2 {
    // --------------------------------
    // Create Primitive Rules
    // --------------------------------

    let nft_amount = Amount::serialize(
        1,
        mpl_token_auth_rules::bytemuck::CompareOp::Eq,
        PayloadKey::Amount.to_string(),
    )
    .unwrap();

    // Generate some random programs to add to the base lists.
    let random_programs = (0..30).map(|_| Keypair::new().pubkey()).collect::<Vec<_>>();

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

    let delegate_rule = All::serialize(&[&nft_amount, &delegate_program_allow_list]).unwrap();

    let advanced_delegate_rule =
        All::serialize(&[&nft_amount, &advanced_delegate_program_allow_list]).unwrap();

    ComposedRulesV2 {
        transfer_rule,
        delegate_rule,
        advanced_delegate_rule,
    }
}

fn get_royalty_rule_set_v2(owner: Pubkey) -> Vec<u8> {
    // Get transfer and wallet-to-wallet rules.
    let composed_rules = get_composed_rules_v2();

    let mut operations = Vec::new();
    let mut rules = Vec::new();

    // --------------------------------
    // Set up transfer operations
    // --------------------------------

    let transfer_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    operations.push(transfer_transfer_delegate_operation.to_string());
    rules.push(composed_rules.transfer_rule);

    // --------------------------------
    // Setup metadata delegate operations
    // --------------------------------

    let metadata_delegate_authority_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Authority),
    };

    operations.push(metadata_delegate_authority_operation.to_string());
    rules.push(composed_rules.delegate_rule);

    // --------------------------------
    // Setup token delegate operations
    // --------------------------------

    let token_delegate_locked_transfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::LockedTransfer),
    };

    operations.push(token_delegate_locked_transfer_operation.to_string());
    rules.push(composed_rules.advanced_delegate_rule);

    RuleSetV2::serialize(owner, RULE_SET_NAME, operations.as_slice(), &rules).unwrap()
}

async fn create_royalty_rule_set_v2(context: &mut ProgramTestContext) -> Pubkey {
    let royalty_rule_set = get_royalty_rule_set_v2(context.payer.pubkey());

    let clone = royalty_rule_set.clone();
    let rule_set = RuleSetV2::from_bytes(&clone).unwrap();
    println!("{}", rule_set);

    // Put the `RuleSet` on chain.
    create_big_rule_set_v2_on_chain!(
        context,
        royalty_rule_set,
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await
}

// Get the four Composed Rules used in this RuleSet.
fn get_composed_rules_v1() -> ComposedRulesV1 {
    // --------------------------------
    // Create Primitive Rules
    // --------------------------------
    let nft_amount = Rule::Amount {
        field: PayloadKey::Amount.to_string(),
        amount: 1,
        operator: CompareOp::Eq,
    };

    // Generate some random programs to add to the base lists.
    let random_programs = (0..30).map(|_| Keypair::new().pubkey()).collect::<Vec<_>>();

    let multi_field_program_allow_list = Rule::ProgramOwnedList {
        programs: [
            TRANSFER_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
        field: format!(
            "{}|{}|{}",
            PayloadKey::Source.to_string(),
            PayloadKey::Destination.to_string(),
            PayloadKey::Authority.to_string()
        ),
    };

    let delegate_program_allow_list = Rule::ProgramOwnedList {
        programs: [
            DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs.clone(),
        ]
        .concat(),
        field: PayloadKey::Delegate.to_string(),
    };

    let advanced_delegate_program_allow_list = Rule::ProgramOwnedList {
        programs: [
            ADVANCED_DELEGATE_PROGRAM_BASE_ALLOW_LIST.to_vec(),
            random_programs,
        ]
        .concat(),
        field: PayloadKey::Delegate.to_string(),
    };

    // --------------------------------
    // Create Composed Rules from
    // Primitive Rules
    // --------------------------------
    // amount is 1 && (source owner on allow list || dest owner on allow list || authority owner on allow list )
    let transfer_rule = Rule::All {
        rules: vec![nft_amount.clone(), multi_field_program_allow_list],
    };

    let delegate_rule = Rule::All {
        rules: vec![nft_amount.clone(), delegate_program_allow_list],
    };

    let advanced_delegate_rule = Rule::All {
        rules: vec![nft_amount, advanced_delegate_program_allow_list],
    };

    ComposedRulesV1 {
        transfer_rule,
        delegate_rule,
        advanced_delegate_rule,
    }
}

fn get_royalty_rule_set_v1(owner: Pubkey) -> RuleSetV1 {
    // Create a RuleSet.
    let mut royalty_rule_set = RuleSetV1::new(RULE_SET_NAME.to_string(), owner);

    // Get transfer and wallet-to-wallet rules.
    let rules = get_composed_rules_v1();

    // --------------------------------
    // Set up transfer operations
    // --------------------------------

    let transfer_transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
    };

    royalty_rule_set
        .add(
            transfer_transfer_delegate_operation.to_string(),
            rules.transfer_rule.clone(),
        )
        .unwrap();

    // --------------------------------
    // Setup metadata delegate operations
    // --------------------------------

    let metadata_delegate_authority_operation = Operation::Delegate {
        scenario: DelegateScenario::Metadata(MetadataDelegateRole::Authority),
    };

    royalty_rule_set
        .add(
            metadata_delegate_authority_operation.to_string(),
            rules.delegate_rule.clone(),
        )
        .unwrap();

    // --------------------------------
    // Setup token delegate operations
    // --------------------------------

    let token_delegate_locked_transfer_operation = Operation::Delegate {
        scenario: DelegateScenario::Token(TokenDelegateRole::LockedTransfer),
    };

    // --------------------------------
    // NOTE THIS IS THE ONLY OPERATION
    // THAT USES THE ADVANCED DELEGATE
    // RULE.
    // --------------------------------
    royalty_rule_set
        .add(
            token_delegate_locked_transfer_operation.to_string(),
            rules.advanced_delegate_rule,
        )
        .unwrap();

    royalty_rule_set
}

async fn create_royalty_rule_set_v1(context: &mut ProgramTestContext) -> Pubkey {
    let royalty_rule_set = get_royalty_rule_set_v1(context.payer.pubkey());

    print!("Royalty Rule Set: {:#?}", royalty_rule_set);

    // Put the `RuleSet` on chain.
    create_big_rule_set_on_chain!(
        context,
        royalty_rule_set.clone(),
        RULE_SET_NAME.to_string(),
        Some(ADDITIONAL_COMPUTE)
    )
    .await
}

#[tokio::test]
async fn create_rule_set_v1() {
    let mut context = program_test().start_with_context().await;
    let _rule_set_addr = create_royalty_rule_set_v1(&mut context).await;
}

#[tokio::test]
async fn create_rule_set_v2() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set_v2(&mut context).await;

    let rule_set_account = context
        .banks_client
        .get_account_with_commitment(rule_set_addr, CommitmentLevel::Processed)
        .await
        .expect("account not found")
        .expect("account empty");

    let key = u32::from_le_bytes(*array_ref![rule_set_account.data, 0, 4]);
    assert_eq!(key, Key::RuleSetV2 as u32);
}

#[tokio::test]
async fn wallet_to_prog_owned_v1() {
    let mut context = program_test().start_with_context().await;
    let rule_set_addr = create_royalty_rule_set_v1(&mut context).await;

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

    let transfer_delegate_operation = Operation::Transfer {
        scenario: TransferScenario::TransferDelegate,
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
            operation: transfer_delegate_operation.to_string(),
            payload,
            update_rule_state: false,
            rule_set_revision: None,
        })
        .unwrap()
        .instruction();

    // Validate operation.
    process_passing_validate_ix!(&mut context, validate_ix, vec![], Some(ADDITIONAL_COMPUTE)).await;
}
