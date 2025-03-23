use solana_sdk::{
    pubkey::Pubkey,
    instruction::Instruction,
    transaction::Transaction,
    system_program,
};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use tokio::time::{sleep, Duration as TokioDuration};
use serde_json;

use crate::{
    models::{AddReadAuthorityRequest, PreparedTransaction},
    utils, // Updated import
    error::AppError,
};

pub struct SolanaService {
    pub client: RpcClient,
    pub program_id: Pubkey,
    pub admin_pubkey: Pubkey,
}

impl SolanaService {
    pub fn new(client: RpcClient, program_id: &str, admin_pubkey: &str) -> Result<Self, AppError> {
        let program_id = Pubkey::from_str(program_id)
            .map_err(|e| AppError::InternalServerError(format!("Invalid program ID: {}", e)))?;
        let admin_pubkey = Pubkey::from_str(admin_pubkey)
            .map_err(|e| AppError::InternalServerError(format!("Invalid admin public key: {}", e)))?;

        Ok(SolanaService {
            client,
            program_id,
            admin_pubkey,
        })
    }

    async fn get_latest_blockhash_with_retry(&self, retries: u32, delay: std::time::Duration) -> Result<solana_sdk::hash::Hash, AppError> {
        for attempt in 1..=retries {
            match self.client.get_latest_blockhash() {
                Ok(blockhash) => return Ok(blockhash),
                Err(e) => {
                    log::error!("Attempt {}/{}: Failed to get latest blockhash: {}", attempt, retries, e);
                    if attempt == retries {
                        return Err(AppError::SolanaError(format!("Failed to get latest blockhash after {} attempts: {}", retries, e)));
                    }
                    sleep(TokioDuration::from_millis(delay.as_millis() as u64)).await;
                }
            }
        }
        Err(AppError::SolanaError("Failed to get latest blockhash".to_string()))
    }

    pub async fn prepare_add_read_authority(&self, req: &AddReadAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        let new_authority = Pubkey::from_str(&req.new_authority)
            .map_err(|e| AppError::BadRequest(format!("Invalid new authority public key: {}", e)))?;

        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);

        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );

        let mut instruction_data = vec![123, 45, 67, 89, 1, 2, 3, 4];
        instruction_data.extend_from_slice(new_authority.as_ref());

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(self.admin_pubkey, true),
                solana_sdk::instruction::AccountMeta::new(admin_pda, true),
                solana_sdk::instruction::AccountMeta::new(history_pda, true),
                solana_sdk::instruction::AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };

        let recent_blockhash = self.get_latest_blockhash_with_retry(3, std::time::Duration::from_secs(2)).await?;

        let transaction = Transaction::new_with_payer(&[instruction], Some(&self.admin_pubkey));

        let serialized_transaction = utils::serialize_transaction(&transaction)?; // Updated reference

        let metadata = serde_json::to_string(req)
            .map_err(|e| AppError::InternalServerError(format!("Failed to serialize metadata: {}", e)))?;

        Ok(PreparedTransaction {
            serialized_transaction,
            transaction_type: "add_read_authority".to_string(),
            metadata,
        })
    }

    pub async fn submit_transaction(&self, serialized_transaction: &str) -> Result<String, AppError> {
        let transaction = utils::deserialize_transaction(serialized_transaction)?; // Updated reference

        let signature = self.client.send_and_confirm_transaction(&transaction)
            .map_err(|e| AppError::SolanaError(format!("Failed to submit transaction: {}", e)))?;

        Ok(signature.to_string())
    }
}