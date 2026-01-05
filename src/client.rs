//! Async client helpers for the Squads v4 protocol
//!
//! This module provides high-level async functions for interacting with the Squads protocol.
//! It combines instruction building with RPC calls to make common operations easier.
//!
//! # Features
//! This module is only available with the `async` feature enabled.

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::{
    accounts::{ConfigTransaction, Multisig, Proposal, SpendingLimit, VaultTransaction},
    error::{SquadsError, SquadsResult},
    instructions,
    pda,
    types::{ConfigAction, Member},
};

/// High-level async client for Squads v4 protocol
pub struct SquadsClient {
    /// RPC client for communicating with Solana
    pub rpc: RpcClient,
    /// Program ID to use (defaults to canonical Squads program ID)
    pub program_id: Pubkey,
}

impl SquadsClient {
    /// Create a new SquadsClient with the default program ID
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc: RpcClient::new(rpc_url),
            program_id: crate::program_id(),
        }
    }

    /// Create a new SquadsClient with a custom program ID
    pub fn new_with_program_id(rpc_url: String, program_id: Pubkey) -> Self {
        Self {
            rpc: RpcClient::new(rpc_url),
            program_id,
        }
    }

    /// Create a client with an existing RpcClient
    pub fn from_rpc_client(rpc: RpcClient) -> Self {
        Self {
            rpc,
            program_id: crate::program_id(),
        }
    }

    /// Fetch and deserialize a Multisig account
    pub async fn get_multisig(&self, multisig: &Pubkey) -> SquadsResult<Multisig> {
        let account = self
            .rpc
            .get_account(multisig)
            .await
            .map_err(|e| SquadsError::ClientError(e))?;

        Multisig::try_from_slice(&account.data)
            .map_err(|_| SquadsError::DeserializationError)
    }

    /// Fetch and deserialize a Proposal account
    pub async fn get_proposal(&self, proposal: &Pubkey) -> SquadsResult<Proposal> {
        let account = self
            .rpc
            .get_account(proposal)
            .await
            .map_err(|e| SquadsError::ClientError(e))?;

        Proposal::try_from_slice(&account.data)
            .map_err(|_| SquadsError::DeserializationError)
    }

    /// Fetch and deserialize a VaultTransaction account
    pub async fn get_vault_transaction(
        &self,
        transaction: &Pubkey,
    ) -> SquadsResult<VaultTransaction> {
        let account = self
            .rpc
            .get_account(transaction)
            .await
            .map_err(|e| SquadsError::ClientError(e))?;

        VaultTransaction::try_from_slice(&account.data)
            .map_err(|_| SquadsError::DeserializationError)
    }

    /// Fetch and deserialize a ConfigTransaction account
    pub async fn get_config_transaction(
        &self,
        transaction: &Pubkey,
    ) -> SquadsResult<ConfigTransaction> {
        let account = self
            .rpc
            .get_account(transaction)
            .await
            .map_err(|e| SquadsError::ClientError(e))?;

        ConfigTransaction::try_from_slice(&account.data)
            .map_err(|_| SquadsError::DeserializationError)
    }

    /// Fetch and deserialize a SpendingLimit account
    pub async fn get_spending_limit(&self, spending_limit: &Pubkey) -> SquadsResult<SpendingLimit> {
        let account = self
            .rpc
            .get_account(spending_limit)
            .await
            .map_err(|e| SquadsError::ClientError(e))?;

        SpendingLimit::try_from_slice(&account.data)
            .map_err(|_| SquadsError::DeserializationError)
    }

    /// Get the vault PDA for a multisig
    pub fn get_vault_pda(&self, multisig: &Pubkey, vault_index: u8) -> (Pubkey, u8) {
        pda::get_vault_pda(multisig, vault_index, Some(&self.program_id))
    }

    /// Get the proposal PDA for a transaction
    pub fn get_proposal_pda(&self, multisig: &Pubkey, transaction_index: u64) -> (Pubkey, u8) {
        pda::get_proposal_pda(multisig, transaction_index, Some(&self.program_id))
    }

    /// Get the transaction PDA
    pub fn get_transaction_pda(&self, multisig: &Pubkey, transaction_index: u64) -> (Pubkey, u8) {
        pda::get_transaction_pda(multisig, transaction_index, Some(&self.program_id))
    }

    /// Create a new multisig
    ///
    /// # Arguments
    /// * `create_key` - Keypair for unique multisig PDA derivation
    /// * `creator` - Creator and fee payer
    /// * `threshold` - Approval threshold
    /// * `members` - Initial members
    /// * `time_lock` - Time lock in seconds (0 for no time lock)
    /// * `config_authority` - Optional config authority (None for autonomous)
    /// * `rent_collector` - Optional rent collector
    pub async fn create_multisig(
        &self,
        create_key: &Keypair,
        creator: &Keypair,
        threshold: u16,
        members: Vec<Member>,
        time_lock: u32,
        config_authority: Option<Pubkey>,
        rent_collector: Option<Pubkey>,
    ) -> SquadsResult<Signature> {
        // Validate inputs
        if threshold == 0 {
            return Err(SquadsError::InvalidThreshold);
        }

        let voting_members = members.iter().filter(|m| m.permissions.has_vote()).count();
        if voting_members == 0 {
            return Err(SquadsError::NoVotingMembers);
        }

        if usize::from(threshold) > voting_members {
            return Err(SquadsError::InvalidThreshold);
        }

        // Derive PDAs
        let (multisig_pda, _) = pda::get_multisig_pda(&create_key.pubkey(), Some(&self.program_id));
        let (program_config_pda, _) = pda::get_program_config_pda(Some(&self.program_id));

        // Get program config to find treasury
        let program_config_account = self.rpc.get_account(&program_config_pda).await?;
        let treasury = Pubkey::new_from_array(
            program_config_account.data[40..72]
                .try_into()
                .map_err(|_| SquadsError::InvalidAccountData("Invalid treasury".to_string()))?,
        );

        let args = instructions::MultisigCreateArgsV2 {
            config_authority,
            threshold,
            members,
            time_lock,
            rent_collector,
            memo: None,
        };

        let ix = instructions::multisig_create_v2(
            program_config_pda,
            treasury,
            multisig_pda,
            create_key.pubkey(),
            creator.pubkey(),
            args,
            Some(self.program_id),
        );

        self.send_and_confirm_transaction(&[ix], &[creator, create_key])
            .await
    }

    /// Create a proposal for a transaction
    ///
    /// # Arguments
    /// * `multisig` - Multisig account
    /// * `transaction_index` - Index of the transaction
    /// * `creator` - Proposal creator (must be member)
    /// * `draft` - Whether to create as draft
    pub async fn create_proposal(
        &self,
        multisig: &Pubkey,
        transaction_index: u64,
        creator: &Keypair,
        draft: bool,
    ) -> SquadsResult<Signature> {
        let (proposal_pda, _) = self.get_proposal_pda(multisig, transaction_index);

        let args = instructions::ProposalCreateArgs {
            transaction_index,
            draft,
        };

        let ix = instructions::proposal_create(
            *multisig,
            proposal_pda,
            creator.pubkey(),
            creator.pubkey(),
            args,
            Some(self.program_id),
        );

        self.send_and_confirm_transaction(&[ix], &[creator]).await
    }

    /// Approve a proposal
    pub async fn approve_proposal(
        &self,
        multisig: &Pubkey,
        proposal: &Pubkey,
        member: &Keypair,
    ) -> SquadsResult<Signature> {
        let args = instructions::ProposalVoteArgs { memo: None };

        let ix = instructions::proposal_approve(
            *multisig,
            *proposal,
            member.pubkey(),
            args,
            Some(self.program_id),
        );

        self.send_and_confirm_transaction(&[ix], &[member]).await
    }

    /// Reject a proposal
    pub async fn reject_proposal(
        &self,
        multisig: &Pubkey,
        proposal: &Pubkey,
        member: &Keypair,
    ) -> SquadsResult<Signature> {
        let args = instructions::ProposalVoteArgs { memo: None };

        let ix = instructions::proposal_reject(
            *multisig,
            *proposal,
            member.pubkey(),
            args,
            Some(self.program_id),
        );

        self.send_and_confirm_transaction(&[ix], &[member]).await
    }

    /// Cancel an approved proposal
    pub async fn cancel_proposal(
        &self,
        multisig: &Pubkey,
        proposal: &Pubkey,
        member: &Keypair,
    ) -> SquadsResult<Signature> {
        let args = instructions::ProposalVoteArgs { memo: None };

        let ix = instructions::proposal_cancel(
            *multisig,
            *proposal,
            member.pubkey(),
            args,
            Some(self.program_id),
        );

        self.send_and_confirm_transaction(&[ix], &[member]).await
    }

    /// Create a config transaction
    ///
    /// # Arguments
    /// * `multisig` - Multisig account
    /// * `creator` - Transaction creator
    /// * `actions` - Configuration actions to execute
    pub async fn create_config_transaction(
        &self,
        multisig: &Pubkey,
        creator: &Keypair,
        actions: Vec<ConfigAction>,
    ) -> SquadsResult<(Signature, u64)> {
        // Get current transaction index
        let multisig_account = self.get_multisig(multisig).await?;
        let transaction_index = multisig_account.transaction_index + 1;

        let (transaction_pda, _) = self.get_transaction_pda(multisig, transaction_index);

        let args = instructions::ConfigTransactionCreateArgs {
            actions,
            memo: None,
        };

        let ix = instructions::config_transaction_create(
            *multisig,
            transaction_pda,
            creator.pubkey(),
            creator.pubkey(),
            args,
            Some(self.program_id),
        );

        let sig = self.send_and_confirm_transaction(&[ix], &[creator]).await?;
        Ok((sig, transaction_index))
    }

    /// Execute a vault transaction
    ///
    /// # Arguments
    /// * `multisig` - Multisig account
    /// * `proposal` - Proposal account
    /// * `transaction` - Transaction to execute
    /// * `member` - Member executing (must have Execute permission)
    /// * `remaining_accounts` - Accounts required by the transaction
    pub async fn execute_vault_transaction(
        &self,
        multisig: &Pubkey,
        proposal: &Pubkey,
        transaction: &Pubkey,
        member: &Keypair,
        remaining_accounts: Vec<solana_sdk::instruction::AccountMeta>,
    ) -> SquadsResult<Signature> {
        let ix = instructions::vault_transaction_execute(
            *multisig,
            *proposal,
            *transaction,
            member.pubkey(),
            remaining_accounts,
            Some(self.program_id),
        );

        self.send_and_confirm_transaction(&[ix], &[member]).await
    }

    /// Execute a config transaction
    pub async fn execute_config_transaction(
        &self,
        multisig: &Pubkey,
        proposal: &Pubkey,
        transaction: &Pubkey,
        member: &Keypair,
        spending_limit_accounts: Vec<Pubkey>,
    ) -> SquadsResult<Signature> {
        let ix = instructions::config_transaction_execute(
            *multisig,
            *proposal,
            *transaction,
            member.pubkey(),
            Some(member.pubkey()),
            spending_limit_accounts,
            Some(self.program_id),
        );

        self.send_and_confirm_transaction(&[ix], &[member]).await
    }

    /// Helper function to send and confirm a transaction
    async fn send_and_confirm_transaction(
        &self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> SquadsResult<Signature> {
        let recent_blockhash = self.rpc.get_latest_blockhash().await?;

        let mut transaction = Transaction::new_with_payer(instructions, Some(&signers[0].pubkey()));
        transaction.sign(signers, recent_blockhash);

        let config = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(CommitmentConfig::confirmed().commitment),
            ..Default::default()
        };

        self.rpc
            .send_and_confirm_transaction_with_spinner_and_config(
                &transaction,
                CommitmentConfig::confirmed(),
                config,
            )
            .await
            .map_err(SquadsError::ClientError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SquadsClient::new("https://api.mainnet-beta.solana.com".to_string());
        assert_eq!(client.program_id, crate::program_id());
    }

    #[test]
    fn test_client_with_custom_program_id() {
        let custom_program_id = Pubkey::new_unique();
        let client = SquadsClient::new_with_program_id(
            "https://api.mainnet-beta.solana.com".to_string(),
            custom_program_id,
        );
        assert_eq!(client.program_id, custom_program_id);
    }
}
