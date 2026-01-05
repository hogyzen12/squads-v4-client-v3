//! Diagnostic script to fetch pending transactions for a specific multisig
//! This helps debug why the app isn't showing pending transactions

use squads_v4_client_v3::{
    accounts::{Multisig, Proposal},
    pda,
    types::ProposalStatus,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::str::FromStr;

const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";
const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";

// User's multisig with pending transaction
const MULTISIG_ADDRESS: &str = "jr7P3dmfnR8XBUSAPPJWNNhyaA4eyUvnpHbgBDfwx83";

// User's wallet address (one of the signers)
const WALLET_ADDRESS: &str = "RnGrVx38FRDJUyH6pS6QHFHikbTrs9m1csNiJPWHaZA"; // Replace with actual wallet address

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Diagnosing Pending Transactions ===\n");

    let rpc_client = RpcClient::new_with_commitment(
        RPC_URL.to_string(),
        CommitmentConfig::confirmed()
    );
    
    let program_id = Pubkey::from_str(SQUADS_PROGRAM_ID)?;
    let multisig_address = Pubkey::from_str(MULTISIG_ADDRESS)?;
    let wallet_pubkey = Pubkey::from_str(WALLET_ADDRESS)?;

    println!("Multisig: {}", multisig_address);
    println!("Wallet: {}", wallet_pubkey);
    println!("Program ID: {}\n", program_id);

    // Step 1: Fetch the multisig account
    println!("=== Step 1: Fetching Multisig Account ===");
    let multisig_account = rpc_client.get_account(&multisig_address)?;
    let multisig = Multisig::try_from_slice(&multisig_account.data)?;
    
    println!("Multisig Info:");
    println!("  Threshold: {}", multisig.threshold);
    println!("  Members: {}", multisig.members.len());
    println!("  Transaction Index: {}", multisig.transaction_index);
    println!("  Stale Transaction Index: {}", multisig.stale_transaction_index);
    
    // Check if wallet is a member
    let is_member = multisig.members.iter().any(|m| &m.key == &wallet_pubkey);
    println!("  Wallet is member: {}", is_member);
    
    if !is_member {
        println!("\n⚠️  WARNING: Wallet is not a member of this multisig!");
        println!("Members:");
        for member in &multisig.members {
            println!("    - {}", member.key);
        }
    }

    // Step 2: Scan for pending transactions
    println!("\n=== Step 2: Scanning for Pending Transactions ===");
    
    let mut pending_count = 0;
    let mut total_checked = 0;
    
    // Check the last 20 transactions (wider range than app's 10)
    let start_index = multisig.transaction_index.saturating_sub(20);
    println!("Checking transaction indices {} to {}\n", start_index, multisig.transaction_index);
    
    for tx_index in start_index..=multisig.transaction_index {
        total_checked += 1;
        
        // Derive proposal PDA
        let (proposal_pda, _) = pda::get_proposal_pda(
            &multisig_address,
            tx_index,
            Some(&program_id),
        );
        
        // Try to fetch the proposal
        match rpc_client.get_account(&proposal_pda) {
            Ok(proposal_account) => {
                println!("Transaction #{}: Proposal exists at {}", tx_index, proposal_pda);
                
                // Try to deserialize
                match Proposal::try_from_slice(&proposal_account.data) {
                    Ok(proposal) => {
                        println!("  Status: {:?}", proposal.status);
                        println!("  Approved: {} members", proposal.approved.len());
                        println!("  Rejected: {} members", proposal.rejected.len());
                        
                        // Check if this is active and needs the wallet's approval
                        let is_active = matches!(proposal.status, ProposalStatus::Active { .. });
                        let has_approved = proposal.approved.contains(&wallet_pubkey);
                        let has_rejected = proposal.rejected.contains(&wallet_pubkey);
                        
                        println!("  Is Active: {}", is_active);
                        println!("  Wallet Approved: {}", has_approved);
                        println!("  Wallet Rejected: {}", has_rejected);
                        
                        if is_active && !has_approved && !has_rejected {
                            println!("  ✅ THIS IS A PENDING TRANSACTION THAT NEEDS APPROVAL!");
                            pending_count += 1;
                        } else {
                            println!("  ℹ️  This transaction doesn't need action from this wallet");
                        }
                        
                        // Show approved members
                        if !proposal.approved.is_empty() {
                            println!("  Approved members:");
                            for member in &proposal.approved {
                                println!("    - {}", member);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  ❌ Failed to deserialize proposal: {}", e);
                    }
                }
            }
            Err(_) => {
                // Proposal doesn't exist - this is normal
                println!("Transaction #{}: No proposal (transaction may not exist or not yet proposed)", tx_index);
            }
        }
        println!();
    }
    
    println!("=== Summary ===");
    println!("Total transactions checked: {}", total_checked);
    println!("Pending transactions found: {}", pending_count);
    
    if pending_count == 0 {
        println!("\n⚠️  No pending transactions found that need this wallet's approval.");
        println!("\nPossible reasons:");
        println!("1. All transactions have already been approved by this wallet");
        println!("2. The pending transaction is outside the scanned range");
        println!("3. The wallet is not a member of this multisig");
        println!("4. Transactions exist but are not in Active status");
    } else {
        println!("\n✅ Found {} pending transaction(s) that need approval!", pending_count);
    }

    Ok(())
}