//! Mainnet integration test for Squads v4 client
//!
//! This example demonstrates creating a multisig on mainnet and deriving PDAs.
//!
//! Run with: cargo run --example mainnet_integration

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use squads_v4_client_v3::{
    instructions::{self, MultisigCreateArgsV2},
    pda,
    types::Member,
};
use std::{error::Error, str::FromStr};

const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";
const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";
const WALLET_PATH: &str = "/Users/hogyzen12/.config/solana/RnGrVx38FRDJUyH6pS6QHFHikbTrs9m1csNiJPWHaZA.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n=== Squads v4 Mainnet Integration Test ===\n");

    // Load wallet
    println!("Loading wallet from: {}", WALLET_PATH);
    let wallet_data = std::fs::read_to_string(WALLET_PATH)?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let wallet = Keypair::try_from(&wallet_bytes[..])?;
    println!("Wallet pubkey: {}", wallet.pubkey());

    // Setup RPC client
    let rpc_client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    
    // Check wallet balance
    let balance = rpc_client.get_balance(&wallet.pubkey())?;
    println!("Wallet balance: {} SOL\n", balance as f64 / 1_000_000_000.0);

    if balance < 10_000_000 {
        return Err("Insufficient balance. Need at least 0.01 SOL for transaction fees.".into());
    }

    let program_id = Pubkey::from_str(SQUADS_PROGRAM_ID)?;

    // Step 1: Get program config PDA (required for creating multisigs)
    println!("Step 1: Deriving Program Config PDA");
    let (program_config_pda, _) = pda::get_program_config_pda(Some(&program_id));
    println!("Program Config PDA: {}", program_config_pda);
    
    // Fetch program config to get treasury
    let program_config_account = rpc_client.get_account(&program_config_pda)?;
    println!("Program config found, size: {} bytes\n", program_config_account.data.len());

    // Deserialize program config to get treasury
    // ProgramConfig layout: 8 bytes discriminator + 32 bytes authority + 8 bytes fee + 32 bytes treasury
    // Treasury is at offset 48
    let treasury = if program_config_account.data.len() >= 80 {
        Pubkey::try_from(&program_config_account.data[48..80])?
    } else {
        return Err("Program config account data too small".into());
    };
    println!("Treasury from config: {}", treasury);

    // Step 2: Create a new multisig
    println!("\nStep 2: Creating new multisig");
    let create_key = Keypair::new();
    println!("Create key: {}", create_key.pubkey());

    let (multisig_pda, _) = pda::get_multisig_pda(&create_key.pubkey(), Some(&program_id));
    println!("Multisig PDA: {}", multisig_pda);

    // Create multisig with single member (our wallet) and threshold of 1
    let members = vec![Member::new(wallet.pubkey())];
    
    let args = MultisigCreateArgsV2 {
        config_authority: None, // Autonomous multisig
        threshold: 1,
        members,
        time_lock: 0,
        rent_collector: None,
        memo: Some("Test multisig from Rust client".to_string()),
    };

    let create_multisig_ix = instructions::multisig_create_v2(
        program_config_pda,
        treasury,
        multisig_pda,
        create_key.pubkey(),
        wallet.pubkey(),
        args,
        Some(program_id),
    );

    // Send create multisig transaction
    let mut transaction = Transaction::new_with_payer(
        &[create_multisig_ix],
        Some(&wallet.pubkey()),
    );
    
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&wallet, &create_key], recent_blockhash);
    
    println!("Sending create multisig transaction...");
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Multisig created! Signature: {}\n", signature);

    // Step 3: Verify multisig and show all derived PDAs
    println!("Step 3: Verifying multisig and derived PDAs");
    let multisig_account = rpc_client.get_account(&multisig_pda)?;
    println!("✓ Multisig account exists, size: {} bytes", multisig_account.data.len());

    // Derive and show all related PDAs
    let (vault_pda_0, vault_bump_0) = pda::get_vault_pda(&multisig_pda, 0, Some(&program_id));
    println!("\nVault PDA (index 0):");
    println!("  Address: {}", vault_pda_0);
    println!("  Bump: {}", vault_bump_0);

    let transaction_index = 1u64;
    let (transaction_pda, transaction_bump) = pda::get_transaction_pda(&multisig_pda, transaction_index, Some(&program_id));
    println!("\nTransaction PDA (index {}):", transaction_index);
    println!("  Address: {}", transaction_pda);
    println!("  Bump: {}", transaction_bump);

    let (proposal_pda, proposal_bump) = pda::get_proposal_pda(&multisig_pda, transaction_index, Some(&program_id));
    println!("\nProposal PDA (for transaction {}):", transaction_index);
    println!("  Address: {}", proposal_pda);
    println!("  Bump: {}", proposal_bump);

    let (ephemeral_signer_pda_0, ephemeral_bump_0) = pda::get_ephemeral_signer_pda(&transaction_pda, 0, Some(&program_id));
    println!("\nEphemeral Signer PDA (ephemeral_index 0):");
    println!("  Address: {}", ephemeral_signer_pda_0);
    println!("  Bump: {}", ephemeral_bump_0);

    println!("\n=== Integration Test Complete ===");
    println!("\nMultisig created successfully on mainnet!");
    println!("Multisig address: {}", multisig_pda);
    println!("Vault address (index 0): {}", vault_pda_0);
    println!("\nYou can inspect the multisig on Solana Explorer:");
    println!("https://explorer.solana.com/address/{}", multisig_pda);
    println!("\nTo fund the vault, send SOL to:");
    println!("https://explorer.solana.com/address/{}", vault_pda_0);
    
    println!("\n✓ Squads v4 client library is working correctly on mainnet!");

    Ok(())
}