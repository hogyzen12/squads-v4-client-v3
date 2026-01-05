//! Program Derived Address (PDA) utilities for Squads v4 protocol
//!
//! This module provides helper functions for deriving PDAs used by the Squads multisig program.
//! PDAs are deterministic addresses derived from seeds and the program ID.

use solana_sdk::pubkey::Pubkey;

use crate::seeds::*;

/// Get the program config PDA
///
/// # Arguments
/// * `program_id` - Optional custom program ID (uses canonical ID if None)
///
/// # Returns
/// Tuple of (PDA pubkey, bump seed)
pub fn get_program_config_pda(program_id: Option<&Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_PREFIX, SEED_PROGRAM_CONFIG],
        program_id.unwrap_or(&crate::program_id()),
    )
}

/// Get the multisig PDA for a given create key
///
/// # Arguments
/// * `create_key` - The public key used as the create key for the multisig
/// * `program_id` - Optional custom program ID (uses canonical ID if None)
///
/// # Returns
/// Tuple of (PDA pubkey, bump seed)
pub fn get_multisig_pda(create_key: &Pubkey, program_id: Option<&Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_PREFIX, SEED_MULTISIG, create_key.as_ref()],
        program_id.unwrap_or(&crate::program_id()),
    )
}

/// Get the vault PDA for a multisig
///
/// # Arguments
/// * `multisig_pda` - The multisig account public key
/// * `vault_index` - The index of the vault (0 for default vault)
/// * `program_id` - Optional custom program ID (uses canonical ID if None)
///
/// # Returns
/// Tuple of (PDA pubkey, bump seed)
pub fn get_vault_pda(
    multisig_pda: &Pubkey,
    vault_index: u8,
    program_id: Option<&Pubkey>,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            multisig_pda.as_ref(),
            SEED_VAULT,
            &[vault_index],
        ],
        program_id.unwrap_or(&crate::program_id()),
    )
}

/// Get the transaction PDA for a multisig transaction
///
/// # Arguments
/// * `multisig_pda` - The multisig account public key
/// * `transaction_index` - The index of the transaction
/// * `program_id` - Optional custom program ID (uses canonical ID if None)
///
/// # Returns
/// Tuple of (PDA pubkey, bump seed)
pub fn get_transaction_pda(
    multisig_pda: &Pubkey,
    transaction_index: u64,
    program_id: Option<&Pubkey>,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            multisig_pda.as_ref(),
            SEED_TRANSACTION,
            &transaction_index.to_le_bytes(),
        ],
        program_id.unwrap_or(&crate::program_id()),
    )
}

/// Get the proposal PDA for a multisig transaction
///
/// # Arguments
/// * `multisig_pda` - The multisig account public key
/// * `transaction_index` - The index of the transaction this proposal is for
/// * `program_id` - Optional custom program ID (uses canonical ID if None)
///
/// # Returns
/// Tuple of (PDA pubkey, bump seed)
pub fn get_proposal_pda(
    multisig_pda: &Pubkey,
    transaction_index: u64,
    program_id: Option<&Pubkey>,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            multisig_pda.as_ref(),
            SEED_TRANSACTION,
            &transaction_index.to_le_bytes(),
            SEED_PROPOSAL,
        ],
        program_id.unwrap_or(&crate::program_id()),
    )
}

/// Get the spending limit PDA for a multisig
///
/// # Arguments
/// * `multisig_pda` - The multisig account public key
/// * `create_key` - The public key used as the create key for the spending limit
/// * `program_id` - Optional custom program ID (uses canonical ID if None)
///
/// # Returns
/// Tuple of (PDA pubkey, bump seed)
pub fn get_spending_limit_pda(
    multisig_pda: &Pubkey,
    create_key: &Pubkey,
    program_id: Option<&Pubkey>,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            multisig_pda.as_ref(),
            SEED_SPENDING_LIMIT,
            create_key.as_ref(),
        ],
        program_id.unwrap_or(&crate::program_id()),
    )
}

/// Get the ephemeral signer PDA for a transaction
///
/// # Arguments
/// * `transaction_pda` - The transaction account public key
/// * `ephemeral_signer_index` - The index of the ephemeral signer
/// * `program_id` - Optional custom program ID (uses canonical ID if None)
///
/// # Returns
/// Tuple of (PDA pubkey, bump seed)
pub fn get_ephemeral_signer_pda(
    transaction_pda: &Pubkey,
    ephemeral_signer_index: u8,
    program_id: Option<&Pubkey>,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            transaction_pda.as_ref(),
            SEED_EPHEMERAL_SIGNER,
            &[ephemeral_signer_index],
        ],
        program_id.unwrap_or(&crate::program_id()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multisig_pda_derivation() {
        let create_key = Pubkey::new_unique();
        let (pda, _bump) = get_multisig_pda(&create_key, None);
        assert_ne!(pda, Pubkey::default());
    }

    #[test]
    fn test_vault_pda_derivation() {
        let multisig_pda = Pubkey::new_unique();
        let (pda, _bump) = get_vault_pda(&multisig_pda, 0, None);
        assert_ne!(pda, Pubkey::default());
    }

    #[test]
    fn test_transaction_pda_derivation() {
        let multisig_pda = Pubkey::new_unique();
        let (pda, _bump) = get_transaction_pda(&multisig_pda, 1, None);
        assert_ne!(pda, Pubkey::default());
    }

    #[test]
    fn test_proposal_pda_derivation() {
        let multisig_pda = Pubkey::new_unique();
        let (pda, _bump) = get_proposal_pda(&multisig_pda, 1, None);
        assert_ne!(pda, Pubkey::default());
    }
}