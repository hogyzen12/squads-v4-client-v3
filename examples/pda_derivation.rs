//! Example: PDA Derivation
//!
//! This example demonstrates how to derive various PDAs used in the Squads protocol.
//!
//! To run this example:
//! ```
//! cargo run --example pda_derivation
//! ```

use solana_sdk::pubkey::Pubkey;
use squads_v4_client_v3::pda;

fn main() {
    println!("Squads v4 Client Example: PDA Derivation\n");

    // Generate a random create key
    let create_key = Pubkey::new_unique();
    println!("Create Key: {}\n", create_key);

    // Derive multisig PDA
    let (multisig_pda, multisig_bump) = pda::get_multisig_pda(&create_key, None);
    println!("Multisig PDA:");
    println!("  Address: {}", multisig_pda);
    println!("  Bump: {}\n", multisig_bump);

    // Derive vault PDA (index 0)
    let (vault_pda, vault_bump) = pda::get_vault_pda(&multisig_pda, 0, None);
    println!("Vault PDA (index 0):");
    println!("  Address: {}", vault_pda);
    println!("  Bump: {}\n", vault_bump);

    // Derive transaction PDA (index 1)
    let transaction_index = 1u64;
    let (transaction_pda, transaction_bump) =
        pda::get_transaction_pda(&multisig_pda, transaction_index, None);
    println!("Transaction PDA (index {}):", transaction_index);
    println!("  Address: {}", transaction_pda);
    println!("  Bump: {}\n", transaction_bump);

    // Derive proposal PDA (for transaction index 1)
    let (proposal_pda, proposal_bump) =
        pda::get_proposal_pda(&multisig_pda, transaction_index, None);
    println!("Proposal PDA (for transaction {}):", transaction_index);
    println!("  Address: {}", proposal_pda);
    println!("  Bump: {}\n", proposal_bump);

    // Derive program config PDA
    let (program_config_pda, config_bump) = pda::get_program_config_pda(None);
    println!("Program Config PDA:");
    println!("  Address: {}", program_config_pda);
    println!("  Bump: {}\n", config_bump);

    // Derive spending limit PDA
    let spending_limit_create_key = Pubkey::new_unique();
    let (spending_limit_pda, spending_limit_bump) =
        pda::get_spending_limit_pda(&multisig_pda, &spending_limit_create_key, None);
    println!("Spending Limit PDA:");
    println!("  Address: {}", spending_limit_pda);
    println!("  Bump: {}\n", spending_limit_bump);

    // Derive ephemeral signer PDA
    let (ephemeral_signer_pda, ephemeral_bump) =
        pda::get_ephemeral_signer_pda(&transaction_pda, 0, None);
    println!("Ephemeral Signer PDA (index 0):");
    println!("  Address: {}", ephemeral_signer_pda);
    println!("  Bump: {}", ephemeral_bump);

    println!("\nAll PDAs derived successfully!");
}