//! Examine raw account bytes to understand the on-chain structure

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::str::FromStr;

const RPC_URL: &str = "https://mainnet.helius-rpc.com/?api-key=93812d12-f56f-4624-97c9-9a4d242db974";

// User's multisig
const MULTISIG_ADDRESS: &str = "jr7P3dmfnR8XBUSAPPJWNNhyaA4eyUvnpHbgBDfwx83";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Examining Raw Account Data ===\n");

    let rpc_client = RpcClient::new_with_commitment(
        RPC_URL.to_string(),
        CommitmentConfig::confirmed()
    );
    
    let multisig_address = Pubkey::from_str(MULTISIG_ADDRESS)?;
    println!("Multisig: {}", multisig_address);

    // Fetch the account
    let account = rpc_client.get_account(&multisig_address)?;
    println!("Account found!");
    println!("  Owner: {}", account.owner);
    println!("  Data length: {} bytes", account.data.len());
    println!("  Lamports: {}", account.lamports);
    
    // First 8 bytes are the Anchor discriminator
    println!("\nFirst 8 bytes (discriminator): {:?}", &account.data[..8]);
    
    // Show the next several bytes in chunks
    println!("\nRaw data after discriminator:");
    let data = &account.data[8..];
    
    // Print in 32-byte chunks for readability
    for (i, chunk) in data.chunks(32).enumerate() {
        println!("Bytes {}-{}: {:?}", i*32, i*32 + chunk.len(), chunk);
    }
    
    println!("\nTotal data bytes (after discriminator): {}", data.len());
    
    // Try to manually parse the structure
    println!("\n=== Manual Parsing Attempt ===");
    
    let mut offset = 0;
    
    // create_key (32 bytes)
    if data.len() >= offset + 32 {
        let create_key = Pubkey::try_from(&data[offset..offset+32])?;
        println!("create_key: {}", create_key);
        offset += 32;
    }
    
    // config_authority (32 bytes)
    if data.len() >= offset + 32 {
        let config_authority = Pubkey::try_from(&data[offset..offset+32])?;
        println!("config_authority: {}", config_authority);
        offset += 32;
    }
    
    // threshold (u16 = 2 bytes)
    if data.len() >= offset + 2 {
        let threshold = u16::from_le_bytes([data[offset], data[offset+1]]);
        println!("threshold: {}", threshold);
        offset += 2;
    }
    
    // time_lock (u32 = 4 bytes)
    if data.len() >= offset + 4 {
        let time_lock = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]);
        println!("time_lock: {}", time_lock);
        offset += 4;
    }
    
    // transaction_index (u64 = 8 bytes)
    if data.len() >= offset + 8 {
        let transaction_index = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]
        ]);
        println!("transaction_index: {}", transaction_index);
        offset += 8;
    }
    
    // stale_transaction_index (u64 = 8 bytes)
    if data.len() >= offset + 8 {
        let stale_index = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]
        ]);
        println!("stale_transaction_index: {}", stale_index);
        offset += 8;
    }
    
    // rent_collector (Option<Pubkey> = 1 byte + potentially 32 bytes)
    if data.len() >= offset + 1 {
        let has_rent_collector = data[offset];
        println!("has_rent_collector: {}", has_rent_collector);
        offset += 1;
        
        if has_rent_collector == 1 && data.len() >= offset + 32 {
            let rent_collector = Pubkey::try_from(&data[offset..offset+32])?;
            println!("rent_collector: {}", rent_collector);
            offset += 32;
        }
    }
    
    // bump (u8 = 1 byte)
    if data.len() >= offset + 1 {
        let bump = data[offset];
        println!("bump: {}", bump);
        offset += 1;
    }
    
    // members (Vec<Member> - starts with u32 length)
    if data.len() >= offset + 4 {
        let members_len = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]);
        println!("\nmembers count: {}", members_len);
        offset += 4;
        
        // Each member is Pubkey (32) + Permissions (1 byte for mask)
        for i in 0..members_len {
            if data.len() >= offset + 33 {
                let member_key = Pubkey::try_from(&data[offset..offset+32])?;
                let permissions_mask = data[offset+32];
                println!("  Member {}: {} (permissions: 0b{:08b})", i, member_key, permissions_mask);
                offset += 33;
            }
        }
    }
    
    println!("\nBytes parsed: {}", offset);
    println!("Bytes remaining: {}", data.len() - offset);
    
    if data.len() > offset {
        println!("\nRemaining bytes: {:?}", &data[offset..]);
    }

    Ok(())
}