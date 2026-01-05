//! Complete multisig transaction flow example
//! 
//! This example demonstrates the complete workflow using an existing multisig:
//! 1. Create a vault transaction
//! 2. Get approvals from members (meeting threshold)
//! 3. Execute the transaction

use squads_v4_client_v3::{
    instructions::{
        self, ProposalCreateArgs, ProposalVoteArgs, VaultTransactionCreateArgs,
    },
    message::TransactionMessage,
    pda,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;

const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";
const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";

// Existing multisig from previous test
const EXISTING_MULTISIG: &str = "jr7P3dmfnR8XBUSAPPJWNNhyaA4eyUvnpHbgBDfwx83";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Complete Multisig Transaction Flow ===\n");

    // Load wallets
    println!("Loading wallets...");
    let principal = read_keypair_file("/Users/hogyzen12/.config/solana/RnGrVx38FRDJUyH6pS6QHFHikbTrs9m1csNiJPWHaZA.json")?;
    let member2 = read_keypair_file("/Users/hogyzen12/.config/solana/6tBou5MHL5aWpDy6cgf3wiwGGK2mR8qs68ujtpaoWrf2.json")?;
    let member3 = read_keypair_file("/Users/hogyzen12/.config/solana/worKFoQQH5KzuBnmS3jKKYsJuUi5toCoEp7n4mwRtwa.json")?;

    println!("  Principal (Member 1): {}", principal.pubkey());
    println!("  Member 2: {}", member2.pubkey());
    println!("  Member 3: {}", member3.pubkey());

    let rpc_client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    
    let program_id = Pubkey::from_str(SQUADS_PROGRAM_ID)?;
    let multisig_pda = Pubkey::from_str(EXISTING_MULTISIG)?;

    println!("\nUsing existing multisig: {}", multisig_pda);

    // Get vault
    let (vault_pda, _) = pda::get_vault_pda(&multisig_pda, 0, Some(&program_id));
    println!("Vault address: {}", vault_pda);

    let vault_balance = rpc_client.get_balance(&vault_pda)?;
    println!("Vault balance: {} SOL", vault_balance as f64 / 1_000_000_000.0);

    if vault_balance < 10_000_000 {
        return Err("Vault needs at least 0.01 SOL to complete this test".into());
    }

    // Step 1: Create vault transaction
    println!("\n=== Step 1: Creating Vault Transaction ===");
    
    // Using transaction index 2 since index 1 was created in previous test
    let transaction_index = 2u64;
    
    println!("Using transaction index: {}", transaction_index);
    
    let (transaction_pda, _) = pda::get_transaction_pda(&multisig_pda, transaction_index, Some(&program_id));
    println!("Transaction PDA: {}", transaction_pda);
    
    // Create instruction to send 0.01 SOL from vault to member2
    let transfer_amount = 10_000_000u64; // 0.01 SOL
    let transfer_ix = system_instruction::transfer(&vault_pda, &member2.pubkey(), transfer_amount);
    
    // Compile the transaction message
    let transaction_message = TransactionMessage::try_compile(&vault_pda, &[transfer_ix])?;
    let transaction_message_bytes = borsh::to_vec(&transaction_message)?;
    
    let vault_tx_args = VaultTransactionCreateArgs {
        vault_index: 0,
        ephemeral_signers: 0,
        transaction_message: transaction_message_bytes,
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

    // Step 3: Approve with principal (1/2)
    println!("\n=== Step 3: First Approval (Principal) ===");

    let vote_args = ProposalVoteArgs { memo: None };
    
    let proposal_approve_ix = instructions::proposal_approve(
        multisig_pda,
        proposal_pda,
        principal.pubkey(),
        vote_args.clone(),
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[proposal_approve_ix], Some(&principal.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);

    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Principal approved! Signature: {} (1/2 approvals)", signature);

    // Step 4: Approve with member3 (2/2 - threshold met!)
    println!("\n=== Step 4: Second Approval (Member 3) ===");

    let vote_args = ProposalVoteArgs { memo: None };
    
    let proposal_approve_ix = instructions::proposal_approve(
        multisig_pda,
        proposal_pda,
        member3.pubkey(),
        vote_args,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[proposal_approve_ix], Some(&member3.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&member3], recent_blockhash);

    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Member 3 approved! Signature: {} (2/2 approvals - THRESHOLD MET!)", signature);

    // Step 5: Execute the transaction
    println!("\n=== Step 5: Executing Transaction ===");

    // Build remaining accounts for execution (vault, destination, and system program)
    let remaining_accounts = vec![
        AccountMeta::new(vault_pda, false),
        AccountMeta::new(member2.pubkey(), false),
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
    ];
    
    let vault_transaction_execute_ix = instructions::vault_transaction_execute(
        multisig_pda,
        proposal_pda,
        transaction_pda,
        principal.pubkey(),
        remaining_accounts,
        Some(program_id),
    );

    let mut transaction = Transaction::new_with_payer(&[vault_transaction_execute_ix], Some(&principal.pubkey()));
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[&principal], recent_blockhash);

    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("✓ Transaction executed! Signature: {}", signature);

    // Verify the results
    println!("\n=== Verification ===");
    let vault_balance = rpc_client.get_balance(&vault_pda)?;
    let member2_balance = rpc_client.get_balance(&member2.pubkey())?;
    
    println!("Vault balance after: {} SOL", vault_balance as f64 / 1_000_000_000.0);
    println!("Member 2 balance: {} SOL", member2_balance as f64 / 1_000_000_000.0);

    println!("\n=== Complete Multisig Flow SUCCESS! ===");
    println!("\nSummary:");
    println!("✓ Created vault transaction to send 0.01 SOL");
    println!("✓ Created proposal");
    println!("✓ Got approval from principal (1/2)");
    println!("✓ Got approval from member3 (2/2 - threshold met)");
    println!("✓ Executed transaction successfully");
    println!("\nTransaction: https://explorer.solana.com/tx/{}", signature);
    println!("Multisig: https://explorer.solana.com/address/{}", multisig_pda);
    println!("Vault: https://explorer.solana.com/address/{}", vault_pda);

    Ok(())
}