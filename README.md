# Squads v4 Client

A modern Rust client library for interacting with the Squads v4 multisig protocol on Solana.

## Overview

This library provides a clean, type-safe interface for creating and managing multisig wallets, proposals, and transactions on the Squads protocol. It's built with Solana SDK 2.3.x for compatibility with the latest Solana ecosystem.

## Features

- **Modern Dependencies**: Uses Solana SDK 2.3.x
- **Type-Safe**: Strongly typed interfaces for all Squads protocol operations
- **Async Support**: Optional async client helpers for streamlined workflows
- **PDA Utilities**: Helper functions for deriving program-derived addresses
- **Standalone**: No dependencies on the Anchor program crate
- **Well Tested**: Comprehensive test coverage

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
squads-v4-client = "0.1.0"

# For async support
squads-v4-client = { version = "0.1.0", features = ["async"] }
```

## Quick Start

### Basic PDA Derivation

```rust
use squads_v4_client_v3::pda;
use solana_sdk::pubkey::Pubkey;

// Derive a multisig PDA
let create_key = Pubkey::new_unique();
let (multisig_pda, bump) = pda::get_multisig_pda(&create_key, None);

// Derive a vault PDA
let (vault_pda, vault_bump) = pda::get_vault_pda(&multisig_pda, 0, None);
```

### Building Instructions

```rust
use squads_v4_client_v3::{instructions, types::{Member, Permissions}};
use solana_sdk::pubkey::Pubkey;

// Create a multisig
let args = instructions::MultisigCreateArgsV2 {
    config_authority: None, // Autonomous multisig
    threshold: 2,
    members: vec![
        Member::new(member1_pubkey),
        Member::new(member2_pubkey),
    ],
    time_lock: 0,
    rent_collector: None,
    memo: None,
};

let ix = instructions::multisig_create_v2(
    program_config,
    treasury,
    multisig_pda,
    create_key,
    creator,
    args,
    None, // Use canonical program ID
);
```

### Using the Async Client (requires `async` feature)

```rust
use squads_v4_client_v3::client::SquadsClient;
use squads_v4_client_v3::types::{Member, Permission, Permissions};
use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = SquadsClient::new("https://api.devnet.solana.com".to_string());
    
    // Create a multisig
    let create_key = Keypair::new();
    let creator = Keypair::new();
    
    let members = vec![
        Member::new(creator.pubkey()),
        Member::new(Keypair::new().pubkey()),
    ];
    
    let signature = client.create_multisig(
        &create_key,
        &creator,
        2, // threshold
        members,
        0, // time_lock
        None, // config_authority
        None, // rent_collector
    ).await?;
    
    println!("Multisig created: {}", signature);
    
    Ok(())
}
```

### Fetching Account Data

```rust
use squads_v4_client_v3::client::SquadsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SquadsClient::new("https://api.mainnet-beta.solana.com".to_string());
    
    // Fetch a multisig account
    let multisig = client.get_multisig(&multisig_pubkey).await?;
    println!("Threshold: {}", multisig.threshold);
    println!("Members: {}", multisig.members.len());
    
    // Fetch a proposal
    let proposal = client.get_proposal(&proposal_pubkey).await?;
    println!("Approved: {}", proposal.approved.len());
    
    Ok(())
}
```

## Core Components

### PDA Derivation (`pda`)

Functions for deriving program-derived addresses:
- `get_multisig_pda()` - Multisig account PDA
- `get_vault_pda()` - Vault PDA
- `get_transaction_pda()` - Transaction PDA
- `get_proposal_pda()` - Proposal PDA
- `get_spending_limit_pda()` - Spending limit PDA

### Account Types (`accounts`)

Structs for deserializing on-chain accounts:
- `Multisig` - Main multisig configuration
- `Proposal` - Proposal voting state
- `VaultTransaction` - Vault transaction data
- `ConfigTransaction` - Configuration transaction data
- `SpendingLimit` - Spending limit configuration

### Instructions (`instructions`)

Functions for building Solana instructions:
- `multisig_create_v2()` - Create a multisig
- `proposal_create()` - Create a proposal
- `proposal_approve()` - Approve a proposal
- `proposal_reject()` - Reject a proposal
- `vault_transaction_create()` - Create a vault transaction
- `vault_transaction_execute()` - Execute a vault transaction
- `config_transaction_create()` - Create a config transaction
- `config_transaction_execute()` - Execute a config transaction

### Types (`types`)

Core data types:
- `Member` - Multisig member with permissions
- `Permissions` - Permission flags (Initiate, Vote, Execute)
- `ProposalStatus` - Proposal state enum
- `ConfigAction` - Configuration actions

### Async Client (`client`, requires `async` feature)

High-level async functions for common operations:
- `create_multisig()` - Create a new multisig
- `get_multisig()` - Fetch multisig account
- `create_proposal()` - Create a proposal
- `approve_proposal()` - Approve a proposal
- `execute_vault_transaction()` - Execute a transaction

## Examples

See the `examples/` directory for complete usage examples:
- `create_multisig.rs` - Creating a multisig
- `manage_proposal.rs` - Proposal lifecycle

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### With async feature

```bash
cargo build --features async
cargo test --features async
```

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Resources

- [Squads Protocol Documentation](https://docs.squads.so)
- [Squads v4 GitHub](https://github.com/squads-protocol/v4)
- [Solana Documentation](https://docs.solana.com)