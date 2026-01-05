//! # Squads v4 Client
//!
//! A modern Rust client library for interacting with the Squads v4 multisig protocol on Solana.
//! This library provides a clean, type-safe interface for creating and managing multisig wallets,
//! proposals, and transactions on the Squads protocol.
//!
//! ## Features
//!
//! - **Modern Dependencies**: Uses Solana SDK 2.3.x for compatibility with the latest Solana ecosystem
//! - **Type-Safe**: Strongly typed interfaces for all Squads protocol operations
//! - **Async Support**: Optional async client helpers for streamlined workflows
//! - **PDA Utilities**: Helper functions for deriving program-derived addresses
//! - **Standalone**: No dependencies on the Anchor program crate, making it lightweight and flexible
//!
//! ## Usage
//!
//! ```rust
//! use squads_v4_client_v3::pda;
//! use solana_sdk::pubkey::Pubkey;
//!
//! // Derive a multisig PDA
//! let create_key = Pubkey::new_unique();
//! let (multisig_pda, bump) = pda::get_multisig_pda(&create_key, None);
//! ```

pub mod accounts;
pub mod error;
pub mod instructions;
pub mod message;
pub mod pda;
pub mod types;

#[cfg(feature = "async")]
pub mod client;

// Re-export commonly used types
pub use error::{SquadsError, SquadsResult};
pub use message::{CompiledInstruction, MessageAddressTableLookup, TransactionMessage};
pub use types::{Member, Permission, Permissions};

/// The canonical Squads v4 program ID on mainnet-beta
pub const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";

/// Seed constants for PDA derivation
pub mod seeds {
    pub const SEED_PREFIX: &[u8] = b"multisig";
    pub const SEED_PROGRAM_CONFIG: &[u8] = b"program_config";
    pub const SEED_MULTISIG: &[u8] = b"multisig";
    pub const SEED_VAULT: &[u8] = b"vault";
    pub const SEED_TRANSACTION: &[u8] = b"transaction";
    pub const SEED_PROPOSAL: &[u8] = b"proposal";
    pub const SEED_SPENDING_LIMIT: &[u8] = b"spending_limit";
    pub const SEED_EPHEMERAL_SIGNER: &[u8] = b"ephemeral_signer";
}

/// Returns the canonical Squads v4 program ID
pub fn program_id() -> solana_sdk::pubkey::Pubkey {
    SQUADS_PROGRAM_ID.parse().unwrap()
}
