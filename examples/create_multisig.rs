//! Example: Creating a Squads multisig
//!
//! This example demonstrates how to create a new multisig using the squads-v4-client library.
//!
//! To run this example:
//! ```
//! cargo run --example create_multisig --features async
//! ```

use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
use squads_v4_client_v3::{
    client::SquadsClient,
    pda,
    types::{Member, Permission, Permissions},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Squads v4 Client Example: Create Multisig\n");

    // Initialize client (using devnet)
    let client = SquadsClient::new("https://api.devnet.solana.com".to_string());
    println!("Connected to Solana devnet");

    // Generate keypairs
    let create_key = Keypair::new();
    let creator = Keypair::new();
    let member1 = Keypair::new();
    let member2 = Keypair::new();

    println!("\nGenerated keypairs:");
    println!("  Create Key: {}", create_key.pubkey());
    println!("  Creator: {}", creator.pubkey());
    println!("  Member 1: {}", member1.pubkey());
    println!("  Member 2: {}", member2.pubkey());

    // Derive multisig PDA
    let (multisig_pda, bump) = pda::get_multisig_pda(&create_key.pubkey(), None);
    println!("\nMultisig PDA: {} (bump: {})", multisig_pda, bump);

    // Define multisig members
    let members = vec![
        Member::new(creator.pubkey()),
        Member::new(member1.pubkey()),
        Member::new(member2.pubkey()),
    ];

    println!("\nMultisig configuration:");
    println!("  Threshold: 2 of 3");
    println!("  Time lock: 0 seconds");
    println!("  Config authority: None (autonomous)");
    println!("  Members: 3");

    // Note: In a real scenario, you would need to:
    // 1. Fund the creator account with SOL
    // 2. Airdrop SOL on devnet: solana airdrop 2 <creator-pubkey> --url devnet
    // 3. Then create the multisig

    println!("\n--- Instructions to complete setup ---");
    println!("1. Fund the creator account:");
    println!("   solana airdrop 2 {} --url devnet", creator.pubkey());
    println!("\n2. Create the multisig:");
    println!("   (This would call client.create_multisig() in a funded environment)");

    // Example of what the actual call would look like:
    /*
    let signature = client.create_multisig(
        &create_key,
        &creator,
        2,           // threshold
        members,
        0,           // time_lock
        None,        // config_authority
        None,        // rent_collector
    ).await?;

    println!("\nMultisig created successfully!");
    println!("Transaction: {}", signature);
    println!("Multisig address: {}", multisig_pda);
    */

    // Derive vault PDA (index 0 is the default vault)
    let (vault_pda, vault_bump) = pda::get_vault_pda(&multisig_pda, 0, None);
    println!("\nDefault vault PDA: {} (bump: {})", vault_pda, vault_bump);

    println!("\nExample complete!");
    println!("In a production environment, you would now be able to:");
    println!("  - Create proposals");
    println!("  - Vote on proposals");
    println!("  - Execute approved transactions");

    Ok(())
}