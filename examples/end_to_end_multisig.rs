//! End-to-end multisig workflow demonstration
//!
//! This example demonstrates the complete Squads multisig workflow:
//! 1. Create a 2-of-3 multisig
//! 2. Fund the vault with SOL
//! 3. Create a vault transaction to send SOL
//! 4. Create a proposal
//! 5. Approve with 2 members (meeting threshold)
//! 6. Execute the transaction
//!
//! Run with: cargo run --example end_to_end_multisig

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use squads_v4_client_v3::{
    instructions::{self, MultisigCreateArgsV2, ProposalCreateArgs, ProposalVoteArgs, VaultTransactionCreateArgs},
    message::TransactionMessage,
    pda,
    types::Member,
};
use std::{error::Error, str::FromStr, thread, time::Duration};

const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";
const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";

// Wallet paths
const PRINCIPAL_WALLET: &str = "/Users/hogyzen12/.config/solana/RnGrVx38FRDJUyH6pS6QHFHikbTrs9m1csNiJPWHaZA.json";
const MEMBER2_WALLET: &str = "/Users/hogyzen12/.config/solana/6tBou5MHL5aWpDy6cgf3wiwGGK2mR8qs68ujtpaoWrf2.json";
const MEMBER3_WALLET: &str = "/Users/hogyzen12/.config/solana/worKFoQQH5KzuBnmS3jKKYsJuUi5toCoEp7n4mwRtwa.json";

fn load_keypair(path: &str) -> Result<Keypair, Box<dyn Error>> {
    let wallet_data = std::fs::read_to_string(path)?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;
    Ok(Keypair::try_from(&wallet_bytes[..])?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n=== Squads v4 End-to-End Multisig Workflow ===\n");

    // Load all three wallets
    println!("Loading wallets...");
    let principal = load_keypair(PRINCIPAL_WALLET)?;
    let member2 = load_keypair(MEMBER2_WALLET)?;
    let member3 = load_keypair(MEMBER3_WALLET)?;
    
    println!("  Principal (Member 1): {}", principal.pubkey());
    println!("  Member 2: {}", member2.pubkey());
    println!("  Member 3: {}", member3.pubkey());

    // Setup RPC client
    let rpc_client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    
    // Check principal wallet balance
    let balance = rpc_client.get_balance(&principal.pubkey())?;
    println!("\nPrincipal balance: {} SOL", balance as f64 / 1_000_000_000.0);

    if balance < 40_000_000 {
        return Err("Insufficient balance in principal wallet. Need at least 0.04 SOL.".into());
    }

    let program_id = Pubkey::from_str(SQUADS_PROGRAM_ID)?;

    // Get program config and treasury
    println!("\n=== Step 1: Getting Program Config ===");
    let (program_config_pda, _) = pda::get_program_config_pda(Some(&program_id));
    let program_config_account = rpc_client.get_account(&program_config_pda)?;
    let treasury = Pubkey::try_from(&program_config_account.data[48..80])?;
    println!("Treasury: {}", treasury);

    // Create a 2-of-3 multisig
    println!("\n=== Step 2: Creating 2-of-3 Multisig ===");
    let create_key = Keypair::new();
    let (multisig_pda, _) = pda::get_multisig_pda(&create_key.pubkey(), Some(&program_id));
    
    let members = vec![
        Member::new(principal.pubkey()),
        Member::new(member2.pubkey()),
        Member::new(member3.pubkey()),
    ];
    
    let args = MultisigCreateArgsV2 {
        config_authority: None,
        threshold: 2, // 2-of-3
        members,
        time_lock: 0,
        rent_collector: None,
        memo: Some("2-of-3 multisig test".to_string()),
    };

    let create_multisig_ix = instructions::multisig_create_v2(
        program_config_pda,
        treasury,
        multisig_pda,
        create_key.pubkey(),
        principal.pubkey(),
        args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(
        &[create_multisig_ix],
        Some(&principal.pubkey()),
    );
    
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal, &create_key], recent_blockhash);
    
    println!("Multisig PDA: {}", multisig_pda);
    println!("Sending create multisig transaction...");
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Multisig created! Signature: {}\n", signature);

    // Get vault address
    let (vault_pda, _) = pda::get_vault_pda(&multisig_pda, 0, Some(&program_id));
    println!("Vault address: {}", vault_pda);

    // Fund the vault with 0.02 SOL
    println!("\n=== Step 3: Funding Vault with 0.02 SOL ===");
    let fund_amount = 20_000_000u64; // 0.02 SOL
    let fund_ix = system_instruction::transfer(&principal.pubkey(), &vault_pda, fund_amount);
    
    let mut transaction = Transaction::new_with_payer(&[fund_ix], Some(&principal.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Vault funded! Signature: {}", signature);
    
    // Verify vault balance
    thread::sleep(Duration::from_secs(2));
    let vault_balance = rpc_client.get_balance(&vault_pda)?;
    println!("Vault balance: {} SOL\n", vault_balance as f64 / 1_000_000_000.0);

    // Create vault transaction to send 0.01 SOL to member2
    println!("\n=== Step 4: Creating Vault Transaction ===");
    let transaction_index = 1u64;
    let (transaction_pda, _) = pda::get_transaction_pda(&multisig_pda, transaction_index, Some(&program_id));
    
    let transfer_amount = 10_000_000u64; // 0.01 SOL
    let transfer_ix = system_instruction::transfer(&vault_pda, &member2.pubkey(), transfer_amount);
    
    // Compile the transaction message in Squads format
    let message = TransactionMessage::try_compile(&vault_pda, &[transfer_ix])?;
    let transaction_message = borsh::to_vec(&message)?;
    
    let vault_tx_args = VaultTransactionCreateArgs {
        vault_index: 0,
        ephemeral_signers: 0,
        transaction_message,
        memo: Some("Send 0.01 SOL to member2".to_string()),
    };

    let vault_tx_create_ix = instructions::vault_transaction_create(
        multisig_pda,
        transaction_pda,
        principal.pubkey(),
        principal.pubkey(),
        vault_tx_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(
        &[vault_tx_create_ix],
        Some(&principal.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);
    
    println!("Transaction PDA: {}", transaction_pda);
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Vault transaction created! Signature: {}\n", signature);

    // Create proposal
    println!("\n=== Step 5: Creating Proposal ===");
    let (proposal_pda, _) = pda::get_proposal_pda(&multisig_pda, transaction_index, Some(&program_id));
    
    let proposal_args = ProposalCreateArgs {
        transaction_index,
        draft: false,
    };

    let proposal_create_ix = instructions::proposal_create(
        multisig_pda,
        proposal_pda,
        principal.pubkey(),
        principal.pubkey(),
        proposal_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(
        &[proposal_create_ix],
        Some(&principal.pubkey()),
    );
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);
    
    println!("Proposal PDA: {}", proposal_pda);
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Proposal created! Signature: {}\n", signature);

    // Approve with principal (1/2)
    println!("\n=== Step 6: Approving with Principal (1/2) ===");
    let vote_args = ProposalVoteArgs { memo: None };
    
    let approve_ix = instructions::proposal_approve(
        multisig_pda,
        proposal_pda,
        principal.pubkey(),
        vote_args.clone(),
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[approve_ix], Some(&principal.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Principal approved! Signature: {}", signature);

    // Approve with member3 (2/2 - threshold met!)
    println!("\n=== Step 7: Approving with Member 3 (2/2 - Threshold Met!) ===");
    
    let approve_ix = instructions::proposal_approve(
        multisig_pda,
        proposal_pda,
        member3.pubkey(),
        vote_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[approve_ix], Some(&member3.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&member3], recent_blockhash);
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Member 3 approved! Signature: {}", signature);
    println!("✓ Threshold reached (2/2)!\n");

    // Execute the transaction
    println!("\n=== Step 8: Executing Transaction ===");
    
    // Build remaining accounts for execution
    let remaining_accounts = vec![
        solana_sdk::instruction::AccountMeta::new(vault_pda, true), // Vault as signer
        solana_sdk::instruction::AccountMeta::new(member2.pubkey(), false), // Destination
        solana_sdk::instruction::AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
    ];

    let execute_ix = instructions::vault_transaction_execute(
        multisig_pda,
        proposal_pda,
        transaction_pda,
        principal.pubkey(),
        remaining_accounts,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[execute_ix], Some(&principal.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);
    
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Transaction executed! Signature: {}\n", signature);

    // Verify final balances
    println!("\n=== Step 9: Verifying Final Balances ===");
    thread::sleep(Duration::from_secs(2));
    
    let vault_balance = rpc_client.get_balance(&vault_pda)?;
    let member2_balance = rpc_client.get_balance(&member2.pubkey())?;
    
    println!("Vault balance: {} SOL", vault_balance as f64 / 1_000_000_000.0);
    println!("Member 2 received: 0.01 SOL");
    println!("Member 2 new balance: {} SOL", member2_balance as f64 / 1_000_000_000.0);

    println!("\n=== End-to-End Workflow Complete! ===");
    println!("\nSummary:");
    println!("✓ Created 2-of-3 multisig");
    println!("✓ Funded vault with 0.02 SOL");
    println!("✓ Created transaction to send 0.01 SOL");
    println!("✓ Got 2 approvals (principal + member3)");
    println!("✓ Executed transaction successfully");
    println!("\nMultisig: https://explorer.solana.com/address/{}", multisig_pda);
    println!("Vault: https://explorer.solana.com/address/{}", vault_pda);

    Ok(())
}