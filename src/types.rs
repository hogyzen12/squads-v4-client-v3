//! Core types for the Squads v4 protocol
//!
//! This module defines the fundamental data types used in the Squads multisig protocol,
//! including members, permissions, proposal statuses, and configuration actions.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Permission flags for multisig members
///
/// Members can have combinations of these permissions:
/// - `INITIATE`: Can create proposals
/// - `VOTE`: Can vote on proposals  
/// - `EXECUTE`: Can execute approved proposals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Permission to create/initiate proposals
    Initiate = 1 << 0,
    /// Permission to vote on proposals
    Vote = 1 << 1,
    /// Permission to execute approved proposals
    Execute = 1 << 2,
}

/// Permissions bitmask for a member
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Permissions {
    /// Bitmask of permissions
    pub mask: u8,
}

impl Permissions {
    /// Create permissions from a bitmask
    pub fn from_mask(mask: u8) -> Self {
        Self { mask }
    }

    /// Create permissions from a list of Permission flags
    pub fn from_vec(permissions: &[Permission]) -> Self {
        let mut mask = 0u8;
        for p in permissions {
            mask |= *p as u8;
        }
        Self { mask }
    }

    /// Check if the permissions include the Initiate permission
    pub fn has_initiate(&self) -> bool {
        self.mask & (Permission::Initiate as u8) != 0
    }

    /// Check if the permissions include the Vote permission
    pub fn has_vote(&self) -> bool {
        self.mask & (Permission::Vote as u8) != 0
    }

    /// Check if the permissions include the Execute permission
    pub fn has_execute(&self) -> bool {
        self.mask & (Permission::Execute as u8) != 0
    }

    /// Full permissions (all flags set)
    pub fn full() -> Self {
        Self {
            mask: (Permission::Initiate as u8)
                | (Permission::Vote as u8)
                | (Permission::Execute as u8),
        }
    }

    /// No permissions
    pub fn none() -> Self {
        Self { mask: 0 }
    }
}

/// A member of a multisig
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Member {
    /// Public key of the member
    pub key: Pubkey,
    /// Permissions granted to this member
    pub permissions: Permissions,
}

impl Member {
    /// Create a new member with full permissions
    pub fn new(key: Pubkey) -> Self {
        Self {
            key,
            permissions: Permissions::full(),
        }
    }

    /// Create a new member with specific permissions
    pub fn with_permissions(key: Pubkey, permissions: Permissions) -> Self {
        Self { key, permissions }
    }
}

/// Status of a proposal
/// Each variant includes a timestamp of when the status was set
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal is in draft mode
    Draft { timestamp: i64 },
    /// Proposal is active and can be voted on
    Active { timestamp: i64 },
    /// Proposal has been rejected
    Rejected { timestamp: i64 },
    /// Proposal has been approved
    Approved { timestamp: i64 },
    /// Proposal has been executed
    Executed { timestamp: i64 },
    /// Proposal has been cancelled
    Cancelled { timestamp: i64 },
}

/// Period type for time-based limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Period {
    /// Daily period
    Day,
    /// Weekly period  
    Week,
    /// Monthly period
    Month,
}

/// Actions that can be performed in a config transaction
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum ConfigAction {
    /// Add a new member to the multisig
    AddMember {
        /// The new member to add
        new_member: Member,
    },
    /// Remove a member from the multisig
    RemoveMember {
        /// Public key of the member to remove
        old_member: Pubkey,
    },
    /// Change the approval threshold
    ChangeThreshold {
        /// New threshold value
        new_threshold: u16,
    },
    /// Set the timelock (in seconds)
    SetTimeLock {
        /// New timelock value in seconds
        new_time_lock: u32,
    },
    /// Add a spending limit
    AddSpendingLimit {
        /// Unique key for this spending limit
        create_key: Pubkey,
        /// Vault index this limit applies to
        vault_index: u8,
        /// Token mint (None for SOL)
        mint: Pubkey,
        /// Amount limit
        amount: u64,
        /// Time period for the limit
        period: Period,
        /// Members who can use this limit
        members: Vec<Pubkey>,
        /// Destinations allowed
        destinations: Vec<Pubkey>,
    },
    /// Remove a spending limit
    RemoveSpendingLimit {
        /// Key of the spending limit to remove
        spending_limit: Pubkey,
    },
    /// Set the config authority
    SetConfigAuthority {
        /// New config authority (None to remove)
        new_config_authority: Option<Pubkey>,
    },
    /// Set the rent collector
    SetRentCollector {
        /// New rent collector (None for default)
        new_rent_collector: Option<Pubkey>,
    },
}

/// Small vector type for efficient storage
/// This matches the SmallVec used in the original program
pub type SmallVec<T> = Vec<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions() {
        let perms = Permissions::from_vec(&[Permission::Vote, Permission::Execute]);
        assert!(!perms.has_initiate());
        assert!(perms.has_vote());
        assert!(perms.has_execute());

        let full = Permissions::full();
        assert!(full.has_initiate());
        assert!(full.has_vote());
        assert!(full.has_execute());

        let none = Permissions::none();
        assert!(!none.has_initiate());
        assert!(!none.has_vote());
        assert!(!none.has_execute());
    }

    #[test]
    fn test_member_creation() {
        let key = Pubkey::new_unique();
        let member = Member::new(key);
        assert_eq!(member.key, key);
        assert!(member.permissions.has_initiate());
        assert!(member.permissions.has_vote());
        assert!(member.permissions.has_execute());
    }

    #[test]
    fn test_member_with_permissions() {
        let key = Pubkey::new_unique();
        let perms = Permissions::from_vec(&[Permission::Vote]);
        let member = Member::with_permissions(key, perms);
        assert_eq!(member.key, key);
        assert!(!member.permissions.has_initiate());
        assert!(member.permissions.has_vote());
        assert!(!member.permissions.has_execute());
    }
}