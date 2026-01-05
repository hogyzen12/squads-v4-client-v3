//! Setup script for hardware wallet testing
//! 
//! Creates a new multisig with 2 hardware wallets + 1 software wallet,
//! funds it, and creates a pending transaction for testing hardware signing.

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use squads_v4_client_v3::{
    instructions::{self, MultisigCreateArgsV2, ProposalCreateArgs, VaultTransactionCreateArgs},
    message::TransactionMessage,
    pda,
    types::Member,
};
use std::{error::Error, str::FromStr, thread, time::Duration};

const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";
const RPC_URL: &str = "https://johna-k3cr1v-fast-mainnet.helius-rpc.com";

// Hot wallet that will create and fund the multisig
const HOT_WALLET_PATH: &str = "/Users/hogyzen12/.config/solana/RnGrVx38FRDJUyH6pS6QHFHikbTrs9m1csNiJPWHaZA.json";

// Multisig members
const HARDWARE_WALLET_1: &str = "D7uvPcmK82AnexJXDBw1tnp9s2BBXf1UX5Edjsx8ptUt";
const HARDWARE_WALLET_2: &str = "5LcChTmwCz8Q1eB8aqNhysyfo4jmmU1zTCeSbvPEj8Bm";
const SOFTWARE_WALLET: &str = "RnGrVx38FRDJUyH6pS6QHFHikbTrs9m1csNiJPWHaZA";

fn load_keypair(path: &str) -> Result<Keypair, Box<dyn Error>> {
    let wallet_data = std::fs::read_to_string(path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    Ok(Keypair::try_from(&wallet_bytes[..])?)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n=== Hardware Wallet Test Setup ===\n");

    // Load the hot wallet (will pay for setup)
    println!("Loading hot wallet...");
    let creator = load_keypair(HOT_WALLET_PATH)?;
    println!("Creator: {}", creator.pubkey());

    // Setup RPC client
    let rpc_client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    
    // Check creator balance
    let balance = rpc_client.get_balance(&creator.pubkey())?;
    println!("Creator balance: {} SOL", balance as f64 / 1_000_000_000.0);

    if balance < 50_000_000 {
        return Err("Insufficient balance. Need at least 0.05 SOL.".into());
    }

    let program_id = Pubkey::from_str(SQUADS_PROGRAM_ID)?;

    // Get program config and treasury
    println!("\n=== Getting Program Config ===");
    let (program_config_pda, _) = pda::get_program_config_pda(Some(&program_id));
    let program_config_account = rpc_client.get_account(&program_config_pda)?;
    let treasury = Pubkey::try_from(&program_config_account.data[48..80])?;
    println!("Treasury: {}", treasury);

    // Parse member pubkeys
    let hw1 = Pubkey::from_str(HARDWARE_WALLET_1)?;
    let hw2 = Pubkey::from_str(HARDWARE_WALLET_2)?;
    let sw = Pubkey::from_str(SOFTWARE_WALLET)?;

    println!("\n=== Multisig Members ===");
    println!("Hardware Wallet 1: {}", hw1);
    println!("Hardware Wallet 2: {}", hw2);
    println!("Software Wallet: {}", sw);

    // Create a 2-of-3 multisig
    println!("\n=== Creating 2-of-3 Multisig ===");
    let create_key = Keypair::new();
    let (multisig_pda, _) = pda::get_multisig_pda(&create_key.pubkey(), Some(&program_id));
    
    let members = vec![
        Member::new(hw1),
        Member::new(hw2),
        Member::new(sw),
    ];
    
    let args = MultisigCreateArgsV2 {
        config_authority: None,
        threshold: 2, // 2-of-3
        members,
        time_lock: 0,
        rent_collector: None,
        memo: Some("Hardware wallet test multisig".to_string()),
    };

    let create_multisig_ix = instructions::multisig_create_v2(
        program_config_pda,
        treasury,
        multisig_pda,
        create_key.pubkey(),
        creator.pubkey(),
        args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_multisig_ix],
        Some(&creator.pubkey()),
    );
    
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&creator, &create_key], recent_blockhash);
    
    println!("Multisig PDA: {}", multisig_pda);
    println!("Sending create multisig transaction...");
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Multisig created! Signature: {}\n", signature);

    // Get vault address
    let (vault_pda, _) = pda::get_vault_pda(&multisig_pda, 0, Some(&program_id));
    println!("Vault address: {}", vault_pda);

    // Fund the vault with 0.042 SOL
    println!("\n=== Funding Vault with 0.042 SOL ===");
    let fund_amount = 42_000_000u64; // 0.042 SOL
    let fund_ix = system_instruction::transfer(&creator.pubkey(), &vault_pda, fund_amount);
    
    let mut transaction = Transaction::new_with_payer(&[fund_ix], Some(&creator.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&creator], recent_blockhash);
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Vault funded! Signature: {}", signature);
    
    // Verify vault balance
    thread::sleep(Duration::from_secs(2));
    let vault_balance = rpc_client.get_balance(&vault_pda)?;
    println!("Vault balance: {} SOL\n", vault_balance as f64 / 1_000_000_000.0);

    // Create vault transaction to send 0.0042 SOL to software wallet
    println!("\n=== Creating Vault Transaction ===");
    let transaction_index = 1u64;
    let (transaction_pda, _) = pda::get_transaction_pda(&multisig_pda, transaction_index, Some(&program_id));
    
    let transfer_amount = 4_200_000u64; // 0.0042 SOL
    let transfer_ix = system_instruction::transfer(&vault_pda, &sw, transfer_amount);
    
    // Compile the transaction message in Squads format
    let message = TransactionMessage::try_compile(&vault_pda, &[transfer_ix])?;
    let transaction_message = borsh::to_vec(&message)?;
    
    let vault_tx_args = VaultTransactionCreateArgs {
        vault_index: 0,
        ephemeral_signers: 0,
        transaction_message,
        memo: Some("Send 0.0042 SOL to software wallet".to_string()),
    };

    let vault_tx_create_ix = instructions::vault_transaction_create(
        multisig_pda,
        transaction_pda,
        creator.pubkey(),
        creator.pubkey(),
        vault_tx_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(
        &[vault_tx_create_ix],
        Some(&creator.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&creator], recent_blockhash);
    
    println!("Transaction PDA: {}", transaction_pda);
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Vault transaction created! Signature: {}\n", signature);

    // Create proposal (but don't approve - leave it for hardware wallet testing)
    println!("\n=== Creating Proposal ===");
    let (proposal_pda, _) = pda::get_proposal_pda(&multisig_pda, transaction_index, Some(&program_id));
    
    let proposal_args = ProposalCreateArgs {
        transaction_index,
        draft: false,
    };

    let proposal_create_ix = instructions::proposal_create(
        multisig_pda,
        proposal_pda,
        creator.pubkey(),
        creator.pubkey(),
        proposal_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(
        &[proposal_create_ix],
        Some(&creator.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&creator], recent_blockhash);
    
    println!("Proposal PDA: {}", proposal_pda);
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Proposal created! Signature: {}\n", signature);

    println!("\n=== Setup Complete! ===");
    println!("\nMultisig Details:");
    println!("  Address: {}", multisig_pda);
    println!("  Vault: {}", vault_pda);
    println!("  Balance: {} SOL", vault_balance as f64 / 1_000_000_000.0);
    println!("  Threshold: 2 of 3");
    println!("\nPending Transaction:");
    println!("  Index: {}", transaction_index);
    println!("  Amount: 0.0042 SOL");
    println!("  Recipient: {}", sw);
    println!("  Proposal: {}", proposal_pda);
    println!("  Status: PENDING (needs 2 approvals)");
    println!("\nNext Steps:");
    println!("1. Open the app and connect a hardware wallet");
    println!("2. The pending transaction should appear in the UI");
    println!("3. Approve with 2 hardware wallets to reach threshold");
    println!("4. Execute the transaction");
    println!("\nExplorer Links:");
    println!("  Multisig: https://explorer.solana.com/address/{}", multisig_pda);
    println!("  Vault: https://explorer.solana.com/address/{}", vault_pda);

    Ok(())
}