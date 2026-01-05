//! Error types for the Squads v4 client library

use thiserror::Error;

/// Result type for Squads operations
pub type SquadsResult<T> = Result<T, SquadsError>;

/// Errors that can occur when using the Squads v4 client
#[derive(Debug, Error)]
pub enum SquadsError {
    /// Error from the Solana client
    #[error("Solana client error: {0}")]
    ClientError(#[from] solana_client::client_error::ClientError),

    /// Failed to deserialize account data
    #[error("Failed to deserialize account data")]
    DeserializationError,

    /// Failed to serialize data
    #[error("Failed to serialize data: {0}")]
    SerializationError(std::io::Error),

    /// Invalid address lookup table account
    #[error("Invalid address lookup table account")]
    InvalidAddressLookupTableAccount,

    /// Invalid transaction message
    #[error("Invalid transaction message")]
    InvalidTransactionMessage,

    /// Account not found
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Invalid account data
    #[error("Invalid account data: {0}")]
    InvalidAccountData(String),

    /// Invalid program ID
    #[error("Invalid program ID")]
    InvalidProgramId,

    /// Program error
    #[error("Program error: {0}")]
    ProgramError(String),

    /// Invalid permissions
    #[error("Invalid permissions: {0}")]
    InvalidPermissions(String),

    /// Invalid threshold
    #[error("Invalid threshold: must be between 1 and number of voting members")]
    InvalidThreshold,

    /// No voting members
    #[error("At least one member must have voting permissions")]
    NoVotingMembers,
}

impl From<std::io::Error> for SquadsError {
    fn from(err: std::io::Error) -> Self {
        SquadsError::SerializationError(err)
    }
}