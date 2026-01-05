//! Transaction message types and utilities for Squads v4
//!
//! This module provides the custom TransactionMessage format required by the Squads program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::{v0, CompileError},
    pubkey::Pubkey,
};

/// SmallVec with u8 length prefix for Borsh serialization
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmallVecU8<T>(Vec<T>);

impl<T> From<Vec<T>> for SmallVecU8<T> {
    fn from(vec: Vec<T>) -> Self {
        SmallVecU8(vec)
    }
}

impl<T: BorshSerialize> BorshSerialize for SmallVecU8<T> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = self.0.len() as u8;
        len.serialize(writer)?;
        for item in &self.0 {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T: BorshDeserialize> BorshDeserialize for SmallVecU8<T> {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let len = u8::deserialize_reader(reader)? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize_reader(reader)?);
        }
        Ok(SmallVecU8(vec))
    }
}

/// SmallVec with u16 length prefix for Borsh serialization
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmallVecU16<T>(Vec<T>);

impl<T> From<Vec<T>> for SmallVecU16<T> {
    fn from(vec: Vec<T>) -> Self {
        SmallVecU16(vec)
    }
}

impl<T: BorshSerialize> BorshSerialize for SmallVecU16<T> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let len = self.0.len() as u16;
        len.serialize(writer)?;
        for item in &self.0 {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T: BorshDeserialize> BorshDeserialize for SmallVecU16<T> {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let len = u16::deserialize_reader(reader)? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize_reader(reader)?);
        }
        Ok(SmallVecU16(vec))
    }
}

/// Transaction message format used by Squads v4
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct TransactionMessage {
    /// The number of signer pubkeys in the account_keys vec
    pub num_signers: u8,
    /// The number of writable signer pubkeys in the account_keys vec
    pub num_writable_signers: u8,
    /// The number of writable non-signer pubkeys in the account_keys vec
    pub num_writable_non_signers: u8,
    /// The list of unique account public keys (including program IDs)
    pub account_keys: SmallVecU8<Pubkey>,
    /// The list of instructions to execute
    pub instructions: SmallVecU8<CompiledInstruction>,
    /// List of address table lookups (not commonly used)
    pub address_table_lookups: SmallVecU8<MessageAddressTableLookup>,
}

/// Compiled instruction format for Squads messages
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct CompiledInstruction {
    /// Index into the message's account_keys array indicating the program account
    pub program_id_index: u8,
    /// Indices into the message's account_keys array indicating which accounts to pass to the instruction
    pub account_indexes: SmallVecU8<u8>,
    /// Instruction data - uses u16 length prefix to support larger instruction data
    pub data: SmallVecU16<u8>,
}

/// Address table lookup (for versioned transactions with lookup tables)
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct MessageAddressTableLookup {
    /// Address lookup table account key
    pub account_key: Pubkey,
    /// List of indexes used to load writable account addresses
    pub writable_indexes: SmallVecU8<u8>,
    /// List of indexes used to load readonly account addresses
    pub readonly_indexes: SmallVecU8<u8>,
}

impl TransactionMessage {
    /// Compile a list of instructions into a TransactionMessage for the vault
    ///
    /// This uses Solana's v0::Message compilation and converts it to the Squads format.
    ///
    /// # Arguments
    /// * `vault_key` - The vault PDA that will be the payer/signer
    /// * `instructions` - The instructions to include in the transaction
    ///
    /// # Returns
    /// A compiled TransactionMessage ready to be serialized and passed to vault_transaction_create
    pub fn try_compile(
        vault_key: &Pubkey,
        instructions: &[Instruction],
    ) -> Result<Self, CompileError> {
        // Use Solana's v0::Message compilation with a dummy blockhash
        let dummy_blockhash = Hash::default();
        let v0_message = v0::Message::try_compile(
            vault_key,
            instructions,
            &[],
            dummy_blockhash,
        )?;
        
        // Extract the message components
        let header = v0_message.header;
        let account_keys = v0_message.account_keys;
        let instructions = v0_message.instructions;
        
        // Calculate the number of static keys
        let num_static_keys: u8 = account_keys
            .len()
            .try_into()
            .map_err(|_| CompileError::AccountIndexOverflow)?;
        
        // Convert to Squads format
        Ok(TransactionMessage {
            num_signers: header.num_required_signatures,
            num_writable_signers: header.num_required_signatures
                .saturating_sub(header.num_readonly_signed_accounts),
            num_writable_non_signers: num_static_keys
                .saturating_sub(header.num_required_signatures)
                .saturating_sub(header.num_readonly_unsigned_accounts),
            account_keys: SmallVecU8(account_keys),
            instructions: SmallVecU8(
                instructions
                    .into_iter()
                    .map(|ix| CompiledInstruction {
                        program_id_index: ix.program_id_index,
                        account_indexes: SmallVecU8(ix.accounts),
                        data: SmallVecU16(ix.data),
                    })
                    .collect(),
            ),
            address_table_lookups: SmallVecU8(Vec::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_compilation() {
        let vault = Pubkey::new_unique();
        let destination = Pubkey::new_unique();
        
        let transfer_ix = solana_sdk::system_instruction::transfer(&vault, &destination, 1000);
        
        let message = TransactionMessage::try_compile(&vault, &[transfer_ix]).unwrap();
        
        assert_eq!(message.num_signers, 1);
        assert_eq!(message.num_writable_signers, 1);
        assert_eq!(message.instructions.0.len(), 1);
    }
}