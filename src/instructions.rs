//! Instruction builders for the Squads v4 protocol
//!
//! This module provides functions to build Solana instructions for interacting with
//! the Squads multisig program. Each function creates a properly formatted instruction
//! with the correct accounts and instruction data.

use borsh::BorshSerialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_sdk_ids::system_program;

use crate::types::{ConfigAction, Member};

/// Helper function to compute Anchor instruction discriminator
/// Discriminator is the first 8 bytes of SHA256("global:instruction_name")
fn instruction_discriminator(name: &str) -> [u8; 8] {
    use solana_sdk::hash::hash;
    let preimage = format!("global:{}", name);
    let hash_result = hash(preimage.as_bytes());
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash_result.to_bytes()[..8]);
    discriminator
}

/// Arguments for creating a multisig
#[derive(Debug, Clone, BorshSerialize)]
pub struct MultisigCreateArgsV2 {
    /// Config authority (None for autonomous multisig)
    pub config_authority: Option<Pubkey>,
    /// Approval threshold
    pub threshold: u16,
    /// Members of the multisig
    pub members: Vec<Member>,
    /// Time lock in seconds
    pub time_lock: u32,
    /// Rent collector (None to disable rent reclamation)
    pub rent_collector: Option<Pubkey>,
    /// Optional memo for indexing
    pub memo: Option<String>,
}

/// Create a new multisig
///
/// # Arguments
/// * `program_config` - Program config PDA
/// * `treasury` - Treasury account (from program config)
/// * `multisig` - Multisig PDA to create
/// * `create_key` - Unique key for multisig PDA derivation (must be signer)
/// * `creator` - Creator and fee payer
/// * `args` - Multisig creation arguments
/// * `program_id` - Optional custom program ID
pub fn multisig_create_v2(
    program_config: Pubkey,
    treasury: Pubkey,
    multisig: Pubkey,
    create_key: Pubkey,
    creator: Pubkey,
    args: MultisigCreateArgsV2,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new_readonly(program_config, false),
        AccountMeta::new(treasury, false),
        AccountMeta::new(multisig, false),
        AccountMeta::new_readonly(create_key, true),
        AccountMeta::new(creator, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let mut data = instruction_discriminator("multisig_create_v2").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Arguments for creating a proposal
#[derive(Debug, Clone, BorshSerialize)]
pub struct ProposalCreateArgs {
    /// Transaction index this proposal is for
    pub transaction_index: u64,
    /// Whether to create as draft
    pub draft: bool,
}

/// Create a new proposal for a transaction
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `proposal` - Proposal PDA to create
/// * `creator` - Proposal creator (must be member)
/// * `rent_payer` - Rent payer for the proposal account
/// * `args` - Proposal creation arguments
/// * `program_id` - Optional custom program ID
pub fn proposal_create(
    multisig: Pubkey,
    proposal: Pubkey,
    creator: Pubkey,
    rent_payer: Pubkey,
    args: ProposalCreateArgs,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new(proposal, false),
        AccountMeta::new_readonly(creator, true),
        AccountMeta::new(rent_payer, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let mut data = instruction_discriminator("proposal_create").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Arguments for voting on a proposal
#[derive(Debug, Clone, BorshSerialize)]
pub struct ProposalVoteArgs {
    /// Optional memo
    pub memo: Option<String>,
}

/// Approve a proposal
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `proposal` - Proposal to approve
/// * `member` - Member voting (must have Vote permission)
/// * `args` - Vote arguments
/// * `program_id` - Optional custom program ID
pub fn proposal_approve(
    multisig: Pubkey,
    proposal: Pubkey,
    member: Pubkey,
    args: ProposalVoteArgs,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new(member, true),
        AccountMeta::new(proposal, false),
    ];

    let mut data = instruction_discriminator("proposal_approve").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Reject a proposal
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `proposal` - Proposal to reject
/// * `member` - Member voting (must have Vote permission)
/// * `args` - Vote arguments
/// * `program_id` - Optional custom program ID
pub fn proposal_reject(
    multisig: Pubkey,
    proposal: Pubkey,
    member: Pubkey,
    args: ProposalVoteArgs,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new(member, true),
        AccountMeta::new(proposal, false),
    ];

    let mut data = instruction_discriminator("proposal_reject").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Cancel an approved proposal
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `proposal` - Proposal to cancel (must be Approved)
/// * `member` - Member voting (must have Vote permission)
/// * `args` - Vote arguments
/// * `program_id` - Optional custom program ID
pub fn proposal_cancel(
    multisig: Pubkey,
    proposal: Pubkey,
    member: Pubkey,
    args: ProposalVoteArgs,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new(member, true),
        AccountMeta::new(proposal, false),
    ];

    let mut data = instruction_discriminator("proposal_cancel").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Arguments for creating a vault transaction
#[derive(Debug, Clone, BorshSerialize)]
pub struct VaultTransactionCreateArgs {
    /// Vault index
    pub vault_index: u8,
    /// Number of ephemeral signers
    pub ephemeral_signers: u8,
    /// Serialized transaction message
    pub transaction_message: Vec<u8>,
    /// Optional memo
    pub memo: Option<String>,
}

/// Create a new vault transaction
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `transaction` - Transaction PDA to create
/// * `creator` - Transaction creator (must have Initiate permission)
/// * `rent_payer` - Rent payer for the transaction account
/// * `args` - Transaction creation arguments
/// * `program_id` - Optional custom program ID
pub fn vault_transaction_create(
    multisig: Pubkey,
    transaction: Pubkey,
    creator: Pubkey,
    rent_payer: Pubkey,
    args: VaultTransactionCreateArgs,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new(multisig, false),
        AccountMeta::new(transaction, false),
        AccountMeta::new_readonly(creator, true),
        AccountMeta::new(rent_payer, true),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let mut data = instruction_discriminator("vault_transaction_create").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Execute a vault transaction
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `proposal` - Proposal for the transaction (must be Approved)
/// * `transaction` - Transaction to execute
/// * `member` - Member executing (must have Execute permission)
/// * `remaining_accounts` - Accounts required by the transaction (lookup tables + instruction accounts)
/// * `program_id` - Optional custom program ID
pub fn vault_transaction_execute(
    multisig: Pubkey,
    proposal: Pubkey,
    transaction: Pubkey,
    member: Pubkey,
    remaining_accounts: Vec<AccountMeta>,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let mut accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new(proposal, false),
        AccountMeta::new_readonly(transaction, false),
        AccountMeta::new_readonly(member, true),
    ];
    accounts.extend(remaining_accounts);

    let data = instruction_discriminator("vault_transaction_execute").to_vec();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Arguments for creating a config transaction
#[derive(Debug, Clone, BorshSerialize)]
pub struct ConfigTransactionCreateArgs {
    /// Configuration actions to execute
    pub actions: Vec<ConfigAction>,
    /// Optional memo
    pub memo: Option<String>,
}

/// Create a new config transaction
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `transaction` - Config transaction PDA to create
/// * `creator` - Transaction creator
/// * `rent_payer` - Rent payer for the transaction account
/// * `args` - Config transaction creation arguments
/// * `program_id` - Optional custom program ID
pub fn config_transaction_create(
    multisig: Pubkey,
    transaction: Pubkey,
    creator: Pubkey,
    rent_payer: Pubkey,
    args: ConfigTransactionCreateArgs,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new_readonly(creator, true),
        AccountMeta::new(rent_payer, true),
        AccountMeta::new(transaction, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let mut data = instruction_discriminator("config_transaction_create").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Execute a config transaction
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `proposal` - Proposal for the transaction (must be Approved)
/// * `transaction` - Config transaction to execute
/// * `member` - Member executing (must have Execute permission)
/// * `rent_payer` - Optional rent payer for reallocation
/// * `spending_limit_accounts` - Optional spending limit accounts being added/removed
/// * `program_id` - Optional custom program ID
pub fn config_transaction_execute(
    multisig: Pubkey,
    proposal: Pubkey,
    transaction: Pubkey,
    member: Pubkey,
    rent_payer: Option<Pubkey>,
    spending_limit_accounts: Vec<Pubkey>,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let mut accounts = vec![
        AccountMeta::new(multisig, false),
        AccountMeta::new_readonly(member, true),
        AccountMeta::new(proposal, false),
        AccountMeta::new(transaction, false),
    ];

    // Add rent_payer if provided
    if let Some(rent_payer) = rent_payer {
        accounts.push(AccountMeta::new(rent_payer, true));
    } else {
        accounts.push(AccountMeta::new_readonly(program_id, false));
    }

    // Add system_program
    accounts.push(AccountMeta::new_readonly(system_program::ID, false));

    // Add spending limit accounts
    for spending_limit in spending_limit_accounts {
        accounts.push(AccountMeta::new(spending_limit, false));
    }

    let data = instruction_discriminator("config_transaction_execute").to_vec();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Arguments for activating a draft proposal
#[derive(Debug, Clone, BorshSerialize)]
pub struct ProposalActivateArgs {}

/// Activate a draft proposal
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `proposal` - Proposal to activate (must be Draft)
/// * `member` - Member activating
/// * `program_id` - Optional custom program ID
pub fn proposal_activate(
    multisig: Pubkey,
    proposal: Pubkey,
    member: Pubkey,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new(proposal, false),
        AccountMeta::new_readonly(member, true),
    ];

    let data = instruction_discriminator("proposal_activate").to_vec();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

/// Arguments for using a spending limit
#[derive(Debug, Clone, BorshSerialize)]
pub struct SpendingLimitUseArgs {
    /// Amount to transfer
    pub amount: u64,
    /// Token decimals
    pub decimals: u8,
    /// Optional memo
    pub memo: Option<String>,
}

/// Use a spending limit to transfer tokens
///
/// # Arguments
/// * `multisig` - Multisig account
/// * `member` - Member using the limit
/// * `spending_limit` - Spending limit account
/// * `vault` - Vault to transfer from
/// * `destination` - Destination account
/// * `mint` - Optional token mint (None for SOL)
/// * `vault_token_account` - Optional vault token account (for SPL tokens)
/// * `destination_token_account` - Optional destination token account (for SPL tokens)
/// * `token_program` - Optional token program (for SPL tokens)
/// * `args` - Spending limit use arguments
/// * `program_id` - Optional custom program ID
pub fn spending_limit_use(
    multisig: Pubkey,
    member: Pubkey,
    spending_limit: Pubkey,
    vault: Pubkey,
    destination: Pubkey,
    mint: Option<Pubkey>,
    vault_token_account: Option<Pubkey>,
    destination_token_account: Option<Pubkey>,
    token_program: Option<Pubkey>,
    args: SpendingLimitUseArgs,
    program_id: Option<Pubkey>,
) -> Instruction {
    let program_id = program_id.unwrap_or_else(crate::program_id);

    let mut accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new_readonly(member, true),
        AccountMeta::new(spending_limit, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(destination, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    // Add optional accounts
    accounts.push(if let Some(mint) = mint {
        AccountMeta::new_readonly(mint, false)
    } else {
        AccountMeta::new_readonly(program_id, false)
    });

    accounts.push(if let Some(vault_token) = vault_token_account {
        AccountMeta::new(vault_token, false)
    } else {
        AccountMeta::new_readonly(program_id, false)
    });

    accounts.push(if let Some(dest_token) = destination_token_account {
        AccountMeta::new(dest_token, false)
    } else {
        AccountMeta::new_readonly(program_id, false)
    });

    accounts.push(if let Some(token_prog) = token_program {
        AccountMeta::new_readonly(token_prog, false)
    } else {
        AccountMeta::new_readonly(program_id, false)
    });

    let mut data = instruction_discriminator("spending_limit_use").to_vec();
    args.serialize(&mut data).unwrap();

    Instruction {
        program_id,
        accounts,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_discriminator() {
        // Test that discriminator is 8 bytes
        let disc = instruction_discriminator("multisig_create_v2");
        assert_eq!(disc.len(), 8);
    }

    #[test]
    fn test_multisig_create_instruction() {
        let args = MultisigCreateArgsV2 {
            config_authority: None,
            threshold: 2,
            members: vec![],
            time_lock: 0,
            rent_collector: None,
            memo: None,
        };

        let ix = multisig_create_v2(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            args,
            None,
        );

        assert_eq!(ix.accounts.len(), 6);
        assert!(!ix.data.is_empty());
    }
}
