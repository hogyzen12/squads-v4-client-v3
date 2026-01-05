//! On-chain account structures for the Squads v4 protocol
//!
//! This module contains the account data structures that are stored on-chain by the Squads program.
//! These structures can be deserialized from account data fetched from the blockchain.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use std::io::Read;

use crate::types::{ConfigAction, Member, Period, ProposalStatus};

/// The main multisig account that stores configuration and state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Multisig {
    /// Key that is used to seed the multisig PDA
    pub create_key: Pubkey,
    /// The authority that can change the multisig config.
    /// If set to Pubkey::default(), the multisig is autonomous and changes go through voting.
    /// Otherwise, this authority can make config changes directly (controlled multisig).
    pub config_authority: Pubkey,
    /// Threshold for approval (number of votes required)
    pub threshold: u16,
    /// Time lock in seconds that must pass between approval and execution
    pub time_lock: u32,
    /// Last transaction index (0 means no transactions created)
    pub transaction_index: u64,
    /// All transactions up to this index are stale (updated when config changes)
    pub stale_transaction_index: u64,
    /// Address where rent can be reclaimed for closed accounts (None disables rent reclamation)
    pub rent_collector: Option<Pubkey>,
    /// PDA bump seed
    pub bump: u8,
    /// Members of the multisig with their permissions
    pub members: Vec<Member>,
}

impl Multisig {
    /// Deserialize a Multisig account from raw account data
    pub fn try_from_slice(data: &[u8]) -> Result<Self, std::io::Error> {
        // Skip the 8-byte Anchor discriminator
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Account data too short",
            ));
        }
        
        // Manual deserialization to handle on-chain format quirks
        let mut offset = 8; // Skip discriminator
        
        let create_key = Pubkey::try_from(&data[offset..offset+32])
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid create_key"))?;
        offset += 32;
        
        let config_authority = Pubkey::try_from(&data[offset..offset+32])
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid config_authority"))?;
        offset += 32;
        
        let threshold = u16::from_le_bytes([data[offset], data[offset+1]]);
        offset += 2;
        
        let time_lock = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]);
        offset += 4;
        
        let transaction_index = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]
        ]);
        offset += 8;
        
        let stale_transaction_index = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]
        ]);
        offset += 8;
        
        // rent_collector: 1 byte flag + 32 bytes ONLY if flag is 1
        let has_rent_collector = data[offset];
        offset += 1;
        
        let rent_collector = if has_rent_collector == 1 {
            let pk = Pubkey::try_from(&data[offset..offset+32])
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid rent_collector"))?;
            offset += 32;
            Some(pk)
        } else {
            // No padding when None - bump comes immediately after
            None
        };
        
        let bump = data[offset];
        offset += 1;
        
        // Manually deserialize members Vec to handle trailing padding bytes
        // Vec format: u32 length + items
        let members_len = u32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]) as usize;
        offset += 4;
        
        let mut members = Vec::with_capacity(members_len);
        for _ in 0..members_len {
            // Each Member is: Pubkey (32 bytes) + Permissions (1 byte)
            if offset + 33 > data.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Not enough bytes for member",
                ));
            }
            
            let key = Pubkey::try_from(&data[offset..offset+32])
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid member key"))?;
            offset += 32;
            
            let permissions_mask = data[offset];
            offset += 1;
            
            members.push(Member {
                key,
                permissions: crate::types::Permissions::from_mask(permissions_mask),
            });
        }
        
        // Ignore any trailing padding bytes (typically 32 bytes of zeros)
        
        Ok(Self {
            create_key,
            config_authority,
            threshold,
            time_lock,
            transaction_index,
            stale_transaction_index,
            rent_collector,
            bump,
            members,
        })
    }
}

// Minimal Borsh implementations for compatibility
impl BorshSerialize for Multisig {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.create_key.serialize(writer)?;
        self.config_authority.serialize(writer)?;
        self.threshold.serialize(writer)?;
        self.time_lock.serialize(writer)?;
        self.transaction_index.serialize(writer)?;
        self.stale_transaction_index.serialize(writer)?;
        
        match &self.rent_collector {
            Some(pubkey) => {
                1u8.serialize(writer)?;
                pubkey.serialize(writer)?;
            }
            None => {
                0u8.serialize(writer)?;
                writer.write_all(&[0u8; 32])?;
            }
        }
        
        self.bump.serialize(writer)?;
        self.members.serialize(writer)?;
        Ok(())
    }
}

impl BorshDeserialize for Multisig {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        // Use try_from_slice which handles the format properly
        // Note: this assumes discriminator was already skipped
        let full_data = [&[0u8; 8][..], *buf].concat();
        Self::try_from_slice(&full_data)
    }
    
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read all bytes and use try_from_slice
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::try_from_slice(&data)
    }
}

impl Multisig {

    /// Calculate the number of members with voting permission
    pub fn num_voters(&self) -> usize {
        self.members
            .iter()
            .filter(|m| m.permissions.has_vote())
            .count()
    }

    /// Calculate the number of members with initiate permission
    pub fn num_proposers(&self) -> usize {
        self.members
            .iter()
            .filter(|m| m.permissions.has_initiate())
            .count()
    }

    /// Calculate the number of members with execute permission
    pub fn num_executors(&self) -> usize {
        self.members
            .iter()
            .filter(|m| m.permissions.has_execute())
            .count()
    }

    /// Calculate the rejection cutoff (minimum rejections to reject a proposal)
    pub fn cutoff(&self) -> usize {
        self.num_voters()
            .checked_sub(usize::from(self.threshold))
            .unwrap()
            .checked_add(1)
            .unwrap()
    }

    /// Check if a pubkey is a member
    pub fn is_member(&self, pubkey: &Pubkey) -> bool {
        self.members.iter().any(|m| &m.key == pubkey)
    }
}

/// Proposal account that tracks voting status for a transaction
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct Proposal {
    /// The multisig this proposal belongs to
    pub multisig: Pubkey,
    /// Index of the transaction this proposal is for
    pub transaction_index: u64,
    /// Current status of the proposal
    pub status: ProposalStatus,
    /// PDA bump seed
    pub bump: u8,
    /// Members who have approved
    pub approved: Vec<Pubkey>,
    /// Members who have rejected
    pub rejected: Vec<Pubkey>,
    /// Members who have cancelled (only applicable when status is Approved)
    pub cancelled: Vec<Pubkey>,
}

impl Proposal {
    /// Deserialize a Proposal account from raw account data
    pub fn try_from_slice(data: &[u8]) -> Result<Self, std::io::Error> {
        // Skip the 8-byte Anchor discriminator
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Account data too short",
            ));
        }
        Self::deserialize(&mut &data[8..])
    }

    /// Check if a member has approved
    pub fn has_approved(&self, member: &Pubkey) -> bool {
        self.approved.contains(member)
    }

    /// Check if a member has rejected
    pub fn has_rejected(&self, member: &Pubkey) -> bool {
        self.rejected.contains(member)
    }

    /// Check if a member has cancelled
    pub fn has_cancelled(&self, member: &Pubkey) -> bool {
        self.cancelled.contains(member)
    }
}

/// Vault transaction account
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct VaultTransaction {
    /// The multisig this transaction belongs to
    pub multisig: Pubkey,
    /// Creator of the transaction
    pub creator: Pubkey,
    /// Transaction index within the multisig
    pub index: u64,
    /// PDA bump seed
    pub bump: u8,
    /// Vault index this transaction executes from
    pub vault_index: u8,
    /// Vault PDA bump
    pub vault_bump: u8,
    /// Bumps for ephemeral signers (additional PDAs used as signers)
    pub ephemeral_signer_bumps: Vec<u8>,
    /// The transaction message to execute
    pub message: VaultTransactionMessage,
}

impl VaultTransaction {
    /// Deserialize a VaultTransaction account from raw account data
    pub fn try_from_slice(data: &[u8]) -> Result<Self, std::io::Error> {
        // Skip the 8-byte Anchor discriminator
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Account data too short",
            ));
        }
        Self::deserialize(&mut &data[8..])
    }
}

/// Transaction message for vault transactions
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Default)]
pub struct VaultTransactionMessage {
    /// Number of signer pubkeys
    pub num_signers: u8,
    /// Number of writable signer pubkeys
    pub num_writable_signers: u8,
    /// Number of writable non-signer pubkeys
    pub num_writable_non_signers: u8,
    /// Account keys required for the transaction
    pub account_keys: Vec<Pubkey>,
    /// Instructions to execute
    pub instructions: Vec<CompiledInstruction>,
    /// Address table lookups for loading additional accounts
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

impl VaultTransactionMessage {
    /// Get total number of account keys including lookups
    pub fn num_all_account_keys(&self) -> usize {
        let num_from_lookups: usize = self
            .address_table_lookups
            .iter()
            .map(|lookup| lookup.writable_indexes.len() + lookup.readonly_indexes.len())
            .sum();
        
        self.account_keys.len() + num_from_lookups
    }

    /// Check if an account index is a static writable account
    pub fn is_static_writable_index(&self, key_index: usize) -> bool {
        let num_account_keys = self.account_keys.len();
        let num_signers = usize::from(self.num_signers);
        let num_writable_signers = usize::from(self.num_writable_signers);
        let num_writable_non_signers = usize::from(self.num_writable_non_signers);

        if key_index >= num_account_keys {
            return false;
        }

        if key_index < num_writable_signers {
            return true;
        }

        if key_index >= num_signers {
            let index_into_non_signers = key_index.saturating_sub(num_signers);
            return index_into_non_signers < num_writable_non_signers;
        }

        false
    }

    /// Check if an account index is a signer
    pub fn is_signer_index(&self, key_index: usize) -> bool {
        key_index < usize::from(self.num_signers)
    }
}

/// Compiled instruction for vault transactions
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct CompiledInstruction {
    /// Index of the program ID in the account keys
    pub program_id_index: u8,
    /// Indices of accounts to pass to the instruction
    pub account_indexes: Vec<u8>,
    /// Instruction data
    pub data: Vec<u8>,
}

/// Address table lookup for loading additional accounts
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct MessageAddressTableLookup {
    /// Address of the lookup table account
    pub account_key: Pubkey,
    /// Indexes of writable accounts to load
    pub writable_indexes: Vec<u8>,
    /// Indexes of readonly accounts to load
    pub readonly_indexes: Vec<u8>,
}

/// Config transaction account for multisig configuration changes
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ConfigTransaction {
    /// The multisig this config transaction belongs to
    pub multisig: Pubkey,
    /// Creator of the transaction
    pub creator: Pubkey,
    /// Transaction index within the multisig
    pub index: u64,
    /// PDA bump seed
    pub bump: u8,
    /// Configuration actions to execute
    pub actions: Vec<ConfigAction>,
}

impl ConfigTransaction {
    /// Deserialize a ConfigTransaction account from raw account data
    pub fn try_from_slice(data: &[u8]) -> Result<Self, std::io::Error> {
        // Skip the 8-byte Anchor discriminator
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Account data too short",
            ));
        }
        Self::deserialize(&mut &data[8..])
    }
}

/// Program configuration account
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ProgramConfig {
    /// Authority that can update program config
    pub authority: Pubkey,
    /// Fee charged for creating a multisig (in lamports)
    pub multisig_creation_fee: u64,
    /// Treasury account that receives fees
    pub treasury: Pubkey,
}

impl ProgramConfig {
    /// Deserialize a ProgramConfig account from raw account data
    pub fn try_from_slice(data: &[u8]) -> Result<Self, std::io::Error> {
        // Skip the 8-byte Anchor discriminator
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Account data too short",
            ));
        }
        Self::deserialize(&mut &data[8..])
    }
}

/// Spending limit account for controlled token transfers
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct SpendingLimit {
    /// The multisig this spending limit belongs to
    pub multisig: Pubkey,
    /// Unique key for this spending limit
    pub create_key: Pubkey,
    /// Vault index this limit applies to
    pub vault_index: u8,
    /// Token mint (system program ID for SOL)
    pub mint: Pubkey,
    /// Maximum amount that can be spent per period
    pub amount: u64,
    /// Time period for the limit
    pub period: Period,
    /// Members who can use this spending limit
    pub members: Vec<Pubkey>,
    /// Allowed destination addresses
    pub destinations: Vec<Pubkey>,
    /// Amount remaining in the current period
    pub remaining_amount: u64,
    /// Unix timestamp when the current period ends
    pub last_reset: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl SpendingLimit {
    /// Deserialize a SpendingLimit account from raw account data
    pub fn try_from_slice(data: &[u8]) -> Result<Self, std::io::Error> {
        // Skip the 8-byte Anchor discriminator
        if data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Account data too short",
            ));
        }
        Self::deserialize(&mut &data[8..])
    }

    /// Check if a member can use this spending limit
    pub fn can_use(&self, member: &Pubkey) -> bool {
        self.members.contains(member)
    }

    /// Check if a destination is allowed
    pub fn is_destination_allowed(&self, destination: &Pubkey) -> bool {
        self.destinations.is_empty() || self.destinations.contains(destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multisig_calculations() {
        use crate::types::Permissions;
        
        let multisig = Multisig {
            create_key: Pubkey::new_unique(),
            config_authority: Pubkey::default(),
            threshold: 2,
            time_lock: 0,
            transaction_index: 0,
            stale_transaction_index: 0,
            rent_collector: None,
            bump: 255,
            members: vec![
                Member::new(Pubkey::new_unique()),
                Member::new(Pubkey::new_unique()),
                Member::with_permissions(Pubkey::new_unique(), Permissions::from_mask(0)),
            ],
        };

        assert_eq!(multisig.num_voters(), 2);
        assert_eq!(multisig.num_proposers(), 2);
        assert_eq!(multisig.num_executors(), 2);
        assert_eq!(multisig.cutoff(), 1); // 2 - 2 + 1 = 1
    }

    #[test]
    fn test_proposal_vote_checks() {
        let member1 = Pubkey::new_unique();
        let member2 = Pubkey::new_unique();
        
        let proposal = Proposal {
            multisig: Pubkey::new_unique(),
            transaction_index: 1,
            status: ProposalStatus::Active,
            bump: 255,
            approved: vec![member1],
            rejected: vec![member2],
            cancelled: vec![],
        };

        assert!(proposal.has_approved(&member1));
        assert!(!proposal.has_approved(&member2));
        assert!(proposal.has_rejected(&member2));
        assert!(!proposal.has_rejected(&member1));
    }
}