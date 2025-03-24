use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use bincode;

use crate::error::AppError;
use crate::models::{AddReadAuthorityRequest, RemoveReadAuthorityRequest, PreparedTransaction};
use crate::utils;

pub struct TransactionService {
    client: RpcClient,
    admin_keypair: Keypair,
    admin_pubkey: Pubkey,
    program_id: Pubkey,
}

impl TransactionService {
    pub fn new(rpc_url: &str, admin_keypair: Keypair, program_id: &str) -> Result<Self, AppError> {
        let client = RpcClient::new(rpc_url.to_string());
        let admin_pubkey = admin_keypair.pubkey();
        let program_id = Pubkey::from_str(program_id)
            .map_err(|e| AppError::InvalidProgramId(format!("Invalid program ID: {}", e)))?;

        Ok(Self {
            client,
            admin_keypair,
            admin_pubkey,
            program_id,
        })
    }

    pub async fn prepare_add_read_authority(&self, req: &AddReadAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        log::info!("Preparing add_read_authority transaction for new authority: {}", req.new_authority);

        let user_pubkey = Pubkey::from_str(&req.user_pubkey)
            .map_err(|e| {
                log::error!("Invalid user public key: {}", e);
                AppError::BadRequest(format!("Invalid user public key: {}", e))
            })?;

        let new_authority = Pubkey::from_str(&req.new_authority)
            .map_err(|e| {
                log::error!("Invalid new authority public key: {}", e);
                AppError::BadRequest(format!("Invalid new authority public key: {}", e))
            })?;

        log::info!("Deriving admin PDA...");
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        log::info!("Admin PDA: {}", admin_pda);

        log::info!("Deriving history PDA...");
        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );
        log::info!("History PDA: {}", history_pda);

        let mut instruction_data = vec![121, 238, 122, 44, 108, 135, 140, 74]; // Discriminator for add_read_authority
        instruction_data.extend_from_slice(new_authority.as_ref());
        log::info!("Instruction data prepared: {:?}", instruction_data);

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(self.admin_pubkey, true),
                solana_sdk::instruction::AccountMeta::new(admin_pda, false),
                solana_sdk::instruction::AccountMeta::new(history_pda, false),
                solana_sdk::instruction::AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };
        log::info!("Instruction created: {:?}", instruction);

        log::info!("Fetching latest blockhash...");
        let recent_blockhash = self.get_latest_blockhash_with_retry(5, std::time::Duration::from_secs(5)).await?;
        log::info!("Latest blockhash: {:?}", recent_blockhash);

        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
        transaction.message.recent_blockhash = recent_blockhash;
        log::info!("Transaction created: {:?}", transaction);

        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);
        log::info!("Transaction partially signed by admin");

        let serialized_transaction = utils::serialize_transaction(&transaction)?;
        log::info!("Transaction serialized: {}", serialized_transaction);

        let metadata = serde_json::to_string(req)?;
        log::info!("Metadata: {}", metadata);

        Ok(PreparedTransaction {
            serialized_transaction,
            transaction_type: "add_read_authority".to_string(),
            metadata,
        })
    }

    pub async fn prepare_remove_read_authority(&self, req: &RemoveReadAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        log::info!("Preparing remove_read_authority transaction for authority: {}", req.authority_to_remove);
    
        let user_pubkey = Pubkey::from_str(&req.user_pubkey)
            .map_err(|e| {
                log::error!("Invalid user public key: {}", e);
                AppError::BadRequest(format!("Invalid user public key: {}", e))
            })?;
    
        let authority_to_remove = Pubkey::from_str(&req.authority_to_remove)
            .map_err(|e| {
                log::error!("Invalid authority to remove public key: {}", e);
                AppError::BadRequest(format!("Invalid authority to remove public key: {}", e))
            })?;
    
        log::info!("Deriving admin PDA...");
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        log::info!("Admin PDA: {}", admin_pda);
    
        log::info!("Deriving history PDA...");
        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );
        log::info!("History PDA: {}", history_pda);
    
        // Correct discriminator from IDL for remove_read_authority
        let mut instruction_data = vec![184, 21, 123, 83, 88, 34, 159, 122]; // [b8, 15, 7b, 53, 58, 22, 9f, 7a]
        instruction_data.extend_from_slice(authority_to_remove.as_ref());
        log::info!("Instruction data prepared: {:?}", instruction_data);
    
        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(self.admin_pubkey, true),
                solana_sdk::instruction::AccountMeta::new(admin_pda, false),
                solana_sdk::instruction::AccountMeta::new(history_pda, false),
                solana_sdk::instruction::AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };
        log::info!("Instruction created: {:?}", instruction);
    
        log::info!("Fetching latest blockhash...");
        let recent_blockhash = self.get_latest_blockhash_with_retry(5, std::time::Duration::from_secs(5)).await?;
        log::info!("Latest blockhash: {:?}", recent_blockhash);
    
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
        transaction.message.recent_blockhash = recent_blockhash;
        log::info!("Transaction created: {:?}", transaction);
    
        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);
        log::info!("Transaction partially signed by admin");
    
        let serialized_transaction = utils::serialize_transaction(&transaction)?;
        log::info!("Transaction serialized: {}", serialized_transaction);
    
        let metadata = serde_json::to_string(req)?;
        log::info!("Metadata: {}", metadata);
    
        Ok(PreparedTransaction {
            serialized_transaction,
            transaction_type: "remove_read_authority".to_string(),
            metadata,
        })
    }

    

    pub async fn submit_transaction(&self, serialized_transaction: &str) -> Result<String, AppError> {
        log::info!("Submitting transaction: {}", serialized_transaction);

        // Deserialize the transaction
        let transaction_bytes = STANDARD
            .decode(serialized_transaction)
            .map_err(|e| AppError::BadRequest(format!("Failed to decode transaction: {}", e)))?;

        let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)
            .map_err(|e| AppError::BadRequest(format!("Failed to deserialize transaction: {}", e)))?;

        // Fetch a fresh blockhash
        let recent_blockhash = self.get_latest_blockhash_with_retry(5, std::time::Duration::from_secs(5)).await?;
        transaction.message.recent_blockhash = recent_blockhash;
        log::info!("Updated blockhash: {}", recent_blockhash);

        // Get the fee payer (first account in account_keys)
        let user_pubkey = transaction.message.account_keys[0];
        log::info!("Fee payer: {}", user_pubkey);

        // Preserve the user's signature
        let mut user_signature = None;
        for (i, sig) in transaction.signatures.iter().enumerate() {
            let pubkey = transaction.message.account_keys[i];
            if pubkey == user_pubkey && sig != &solana_sdk::signature::Signature::default() {
                user_signature = Some(*sig);
                break;
            }
        }

        // Clear signatures and re-sign with admin keypair
        transaction.signatures = transaction
            .message
            .account_keys
            .iter()
            .map(|pubkey| {
                if *pubkey == user_pubkey {
                    solana_sdk::signature::Signature::default() // Placeholder for user signature
                } else {
                    solana_sdk::signature::Signature::default()
                }
            })
            .collect();

        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);
        log::info!("Transaction re-signed by admin");

        // Restore the user's signature
        if let Some(sig) = user_signature {
            for (i, signature) in transaction.signatures.iter_mut().enumerate() {
                let pubkey = transaction.message.account_keys[i];
                if pubkey == user_pubkey {
                    *signature = sig;
                    break;
                }
            }
        } else {
            return Err(AppError::BadRequest("User signature not found".to_string()));
        }

        // Submit the transaction to the Solana network
        let signature = self
            .client
            .send_and_confirm_transaction(&transaction)
            .await
            .map_err(|e| {
                log::error!("Failed to submit transaction: {}", e);
                AppError::SolanaError(format!("Failed to submit transaction: {}", e))
            })?;

        log::info!("Transaction submitted with signature: {}", signature);
        Ok(signature.to_string())
    }

    async fn get_latest_blockhash_with_retry(
        &self,
        retries: u32,
        delay: std::time::Duration,
    ) -> Result<solana_sdk::hash::Hash, AppError> {
        for attempt in 1..=retries {
            match self.client.get_latest_blockhash().await {
                Ok(blockhash) => return Ok(blockhash),
                Err(e) => {
                    log::error!(
                        "Attempt {}/{}: Failed to get latest blockhash: {}",
                        attempt,
                        retries,
                        e
                    );
                    if attempt == retries {
                        return Err(AppError::SolanaError(format!(
                            "Failed to get latest blockhash after {} attempts: {}",
                            retries, e
                        )));
                    }
                    sleep(Duration::from_millis(delay.as_millis() as u64)).await;
                }
            }
        }
        Err(AppError::SolanaError(
            "Failed to get latest blockhash".to_string(),
        ))
    }
}