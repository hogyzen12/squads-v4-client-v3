//! Test fetching the pending transaction we created
//! This will help debug the "Not all bytes read" error

use squads_v4_client_v3::accounts::Proposal;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::str::FromStr;

const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";

// The proposal we created
const PROPOSAL_PDA: &str = "D5AsLKNt1jaYnSCQJxidgHFYzrEM6H8ToFGiJA5AWBG1";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Proposal Fetch ===\n");

    let rpc_client = RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    
    let proposal_pda = Pubkey::from_str(PROPOSAL_PDA)?;
    println!("Fetching proposal: {}", proposal_pda);

    // Fetch the account
    let account = rpc_client.get_account(&proposal_pda)?;
    println!("Account found!");
    println!("  Data length: {} bytes", account.data.len());
    println!("  Owner: {}", account.owner);
    
    // Try to deserialize
    println!("\nAttempting to deserialize proposal...");
    
    // Skip the 8-byte discriminator
    let account_data = &account.data[8..];
    println!("  Data after discriminator: {} bytes", account_data.len());
    
    match borsh::from_slice::<Proposal>(account_data) {
        Ok(proposal) => {
            println!("\n✓ Successfully deserialized proposal!");
            println!("\nProposal Details:");
            println!("  Multisig: {}", proposal.multisig);
            println!("  Transaction Index: {}", proposal.transaction_index);
            println!("  Status: {:?}", proposal.status);
            println!("  Approved: {} members", proposal.approved.len());
            println!("  Rejected: {} members", proposal.rejected.len());
            println!("  Cancelled: {} members", proposal.cancelled.len());
            
            println!("\nApproved members:");
            for member in &proposal.approved {
                println!("    - {}", member);
            }
            
            println!("\n✓ Proposal fetch and deserialization successful!");
        }
        Err(e) => {
            println!("\n✗ Deserialization failed!");
            println!("  Error: {}", e);
            println!("\nThis is the same error the app is experiencing.");
            
            // Try to get more details
            println!("\nDebug info:");
            println!("  First 32 bytes of data: {:?}", &account_data[..32.min(account_data.len())]);
        }
    }

    Ok(())
}