//! Debug multisig deserialization to find the exact issue

use squads_v4_client_v3::accounts::Multisig;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::str::FromStr;

const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";
const MULTISIG_ADDRESS: &str = "jr7P3dmfnR8XBUSAPPJWNNhyaA4eyUvnpHbgBDfwx83";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Debug Multisig Deserialization ===\n");

    let rpc_client = RpcClient::new_with_commitment(
        RPC_URL.to_string(),
        CommitmentConfig::confirmed()
    );
    
    let multisig_address = Pubkey::from_str(MULTISIG_ADDRESS)?;
    println!("Fetching account: {}", multisig_address);

    let account = rpc_client.get_account(&multisig_address)?;
    println!("Account data length: {} bytes\n", account.data.len());
    
    // Try to deserialize
    match Multisig::try_from_slice(&account.data) {
        Ok(multisig) => {
            println!("SUCCESS! Multisig deserialized:");
            println!("  Create key: {}", multisig.create_key);
            println!("  Config authority: {}", multisig.config_authority);
            println!("  Threshold: {}", multisig.threshold);
            println!("  Time lock: {}", multisig.time_lock);
            println!("  Transaction index: {}", multisig.transaction_index);
            println!("  Stale transaction index: {}", multisig.stale_transaction_index);
            println!("  Rent collector: {:?}", multisig.rent_collector);
            println!("  Bump: {}", multisig.bump);
            println!("  Members: {}", multisig.members.len());
            for (i, member) in multisig.members.iter().enumerate() {
                println!("    Member {}: {}", i, member.key);
            }
        }
        Err(e) => {
            println!("FAILED to deserialize: {:?}", e);
            
            // Manual step-by-step to find where it fails
            let data = &account.data;
            let mut offset = 8;
            
            println!("\nStep-by-step parsing:");
            println!("  Offset after discriminator: {}", offset);
            println!("  Available bytes: {}", data.len() - offset);
            
            offset += 32; // create_key
            println!("  After create_key: offset={}, available={}", offset, data.len() - offset);
            
            offset += 32; // config_authority  
            println!("  After config_authority: offset={}, available={}", offset, data.len() - offset);
            
            offset += 2 + 4 + 8 + 8; // threshold + time_lock + tx_index + stale_tx_index
            println!("  After numeric fields: offset={}, available={}", offset, data.len() - offset);
            
            offset += 1 + 32; // has_rent_collector + padding
            println!("  After rent_collector: offset={}, available={}", offset, data.len() - offset);
            
            offset += 1; // bump
            println!("  After bump: offset={}, available={}", offset, data.len() - offset);
            
            let members_len = u32::from_le_bytes([
                data[offset], data[offset+1], data[offset+2], data[offset+3]
            ]) as usize;
            offset += 4;
            println!("  Members count: {}", members_len);
            println!("  After members_len: offset={}, available={}", offset, data.len() - offset);
            println!("  Need {} bytes for {} members ({} bytes each)", members_len * 33, members_len, 33);
            
            for i in 0..members_len {
                println!("  Parsing member {}: offset={}, need 33 bytes, available={}", 
                    i, offset, data.len() - offset);
                if offset + 33 > data.len() {
                    println!("    ERROR: Not enough bytes!");
                    break;
                }
                offset += 33;
            }
            
            println!("\n  Final offset: {}", offset);
            println!("  Remaining bytes: {}", data.len() - offset);
        }
    }

    Ok(())
}