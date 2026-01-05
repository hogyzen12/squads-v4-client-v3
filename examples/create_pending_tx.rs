//! Create a pending transaction for testing the app
//! This creates a vault transaction and proposal but doesn't approve it

use squads_v4_client_v3::{
    instructions::{
        self, ProposalCreateArgs, VaultTransactionCreateArgs,
    },
    message::TransactionMessage,
    pda,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;

const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";
const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";

// Existing multisig with transaction index 2
const EXISTING_MULTISIG: &str = "jr7P3dmfnR8XBUSAPPJWNNhyaA4eyUvnpHbgBDfwx83";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Creating Pending Transaction for Testing ===\n");

    // Load wallet
    let principal = read_keypair_file("/Users/hogyzen12/.config/solana/RnGrVx38FRDJUyH6pS6QHFHikbTrs9m1csNiJPWHaZA.json")?;
    let member2 = read_keypair_file("/Users/hogyzen12/.config/solana/6tBou5MHL5aWpDy6cgf3wiwGGK2mR8qs68ujtpaoWrf2.json")?;

    println!("Principal: {}", principal.pubkey());
    println!("Member 2: {}", member2.pubkey());

    let rpc_client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    
    let program_id = Pubkey::from_str(SQUADS_PROGRAM_ID)?;
    let multisig_pda = Pubkey::from_str(EXISTING_MULTISIG)?;

    println!("\nUsing multisig: {}", multisig_pda);

    // Get vault
    let (vault_pda, _) = pda::get_vault_pda(&multisig_pda, 0, Some(&program_id));
    println!("Vault address: {}", vault_pda);

    let vault_balance = rpc_client.get_balance(&vault_pda)?;
    println!("Vault balance: {} SOL", vault_balance as f64 / 1_000_000_000.0);

    // Fund vault if needed
    if vault_balance < 20_000_000 {
        println!("\n=== Funding Vault ===");
        let fund_amount = 20_000_000u64; // 0.02 SOL
        let fund_ix = system_instruction::transfer(&principal.pubkey(), &vault_pda, fund_amount);
        
        let mut transaction = Transaction::new_with_payer(&[fund_ix], Some(&principal.pubkey()));
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
        transaction.sign(&[&principal], recent_blockhash);
        
        let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
        println!("✓ Vault funded! Signature: {}", signature);
        
        let new_balance = rpc_client.get_balance(&vault_pda)?;
        println!("New vault balance: {} SOL", new_balance as f64 / 1_000_000_000.0);
    }

    // Step 1: Create vault transaction at index 3 (current index is 2)
    println!("\n=== Step 1: Creating Vault Transaction ===");
    
    let transaction_index = 3u64;
    println!("Using transaction index: {}", transaction_index);
    
    let (transaction_pda, _) = pda::get_transaction_pda(&multisig_pda, transaction_index, Some(&program_id));
    println!("Transaction PDA: {}", transaction_pda);
    
    // Create instruction to send 0.001 SOL from vault to member2
    let transfer_amount = 1_000_000u64; // 0.001 SOL
    let transfer_ix = system_instruction::transfer(&vault_pda, &member2.pubkey(), transfer_amount);
    
    // Compile the transaction message
    let transaction_message = TransactionMessage::try_compile(&vault_pda, &[transfer_ix])?;
    let transaction_message_bytes = borsh::to_vec(&transaction_message)?;
    
    let vault_tx_args = VaultTransactionCreateArgs {
        vault_index: 0,
        ephemeral_signers: 0,
        transaction_message: transaction_message_bytes,
        memo: Some("Test transaction - 0.001 SOL to member2".to_string()),
    };

    let vault_tx_create_ix = instructions::vault_transaction_create(
        multisig_pda,
        transaction_pda,
        principal.pubkey(),
        principal.pubkey(),
        vault_tx_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[vault_tx_create_ix], Some(&principal.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);

    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Vault transaction created! Signature: {}", signature);

    // Step 2: Create proposal for the transaction
    println!("\n=== Step 2: Creating Proposal ===");
    
    let (proposal_pda, _) = pda::get_proposal_pda(&multisig_pda, transaction_index, Some(&program_id));
    println!("Proposal PDA: {}", proposal_pda);

    let proposal_create_args = ProposalCreateArgs {
        transaction_index,
        draft: false,
    };

    let proposal_create_ix = instructions::proposal_create(
        multisig_pda,
        proposal_pda,
        principal.pubkey(),
        principal.pubkey(),
        proposal_create_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[proposal_create_ix], Some(&principal.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);

    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Proposal created! Signature: {}", signature);

    println!("\n=== SUCCESS! ===");
    println!("\nPending transaction created:");
    println!("  Multisig: {}", multisig_pda);
    println!("  Transaction Index: {}", transaction_index);
    println!("  Transaction PDA: {}", transaction_pda);
    println!("  Proposal PDA: {}", proposal_pda);
    println!("  Amount: 0.001 SOL to {}", member2.pubkey());
    println!("  Status: Pending (0/2 approvals)");
    println!("\nThis transaction is now pending and should appear in the app!");
    println!("View on explorer:");
    println!("  Multisig: https://explorer.solana.com/address/{}", multisig_pda);
    println!("  Proposal: https://explorer.solana.com/address/{}", proposal_pda);

    Ok(())
}