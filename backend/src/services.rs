use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
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
use borsh::BorshDeserialize;
use crate::error::AppError;
use crate::models::{AddReadAuthorityRequest, RemoveReadAuthorityRequest, AddWriteAuthorityRequest, RemoveWriteAuthorityRequest, PreparedTransaction, AuthoritiesResponse, CreatePatientRequest, PreparedPatientTransaction};
use crate::utils;
use rand::Rng;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key
};

pub struct TransactionService {
    client: RpcClient,
    admin_keypair: Keypair,
    admin_pubkey: Pubkey,
    program_id: Pubkey,
    encryption_key: [u8; 32], // Changed to raw bytes
}

impl TransactionService {
    pub fn new(rpc_url: &str, admin_keypair: Keypair, program_id: &str) -> Result<Self, AppError> {
        let client = RpcClient::new(rpc_url.to_string());
        let admin_pubkey = admin_keypair.pubkey();
        let program_id = Pubkey::from_str(program_id)
            .map_err(|e| AppError::InvalidProgramId(format!("Invalid program ID: {}", e)))?;
        
        let encryption_key = rand::thread_rng().gen::<[u8; 32]>();

        Ok(Self {
            client,
            admin_keypair,
            admin_pubkey,
            program_id,
            encryption_key,
        })
    }

    pub async fn prepare_create_patient(&self, req: &CreatePatientRequest) -> Result<PreparedPatientTransaction, AppError> {
        log::info!("Preparing create_patient transaction for user: {}", req.user_pubkey);

        let user_pubkey = Pubkey::from_str(&req.user_pubkey)
            .map_err(|e| AppError::BadRequest(format!("Invalid user public key: {}", e)))?;

        let patient_seed = Keypair::new();
        let patient_seed_pubkey = patient_seed.pubkey();

        let (patient_pda, _bump) = Pubkey::find_program_address(
            &[b"patient", user_pubkey.as_ref(), patient_seed_pubkey.as_ref()],
            &self.program_id,
        );

        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);

        let nonce_bytes = rand::thread_rng().gen::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes); // Bind to a variable
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let encrypted_data = cipher.encrypt(nonce, req.patient_data.as_bytes())
            .map_err(|e| AppError::InternalServerError(format!("Encryption failed: {}", e)))?;
        let encrypted_data_base64 = STANDARD.encode(&encrypted_data);
        let nonce_base64 = STANDARD.encode(&nonce_bytes);
        let encrypted_string = format!("{}|{}", encrypted_data_base64, nonce_base64);

        let discriminator = [176, 85, 210, 156, 179, 74, 60, 203]; // From IDL

        let encrypted_data_bytes = encrypted_string.as_bytes();
        let mut instruction_data = Vec::with_capacity(8 + 4 + encrypted_data_bytes.len());
        instruction_data.extend_from_slice(&discriminator);
        instruction_data.extend_from_slice(&(encrypted_data_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(encrypted_data_bytes);

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(patient_pda, false),
                AccountMeta::new_readonly(patient_seed_pubkey, false),
                AccountMeta::new_readonly(user_pubkey, true),
                AccountMeta::new_readonly(admin_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };

        let recent_blockhash = self.get_latest_blockhash_with_retry(5, Duration::from_secs(5)).await?;
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
        transaction.message.recent_blockhash = recent_blockhash;

        let serialized_transaction = utils::serialize_transaction(&transaction)?;
        let encrypted_data_with_seed = format!("{}|{}", encrypted_string, patient_seed_pubkey.to_string());

        Ok(PreparedPatientTransaction {
            serialized_transaction,
            transaction_type: "create_patient".to_string(),
            encrypted_data_with_seed,
        })
    }

    pub async fn prepare_add_read_authority(&self, req: &AddReadAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        log::info!("Preparing add_read_authority transaction for new authority: {}", req.new_authority);

        let user_pubkey = Pubkey::from_str(&req.user_pubkey)
            .map_err(|e| AppError::BadRequest(format!("Invalid user public key: {}", e)))?;

        let new_authority = Pubkey::from_str(&req.new_authority)
            .map_err(|e| AppError::BadRequest(format!("Invalid new authority public key: {}", e)))?;

        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );

        let mut instruction_data = vec![121, 238, 122, 44, 108, 135, 140, 74];
        instruction_data.extend_from_slice(new_authority.as_ref());

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin_pubkey, true),
                AccountMeta::new(admin_pda, false),
                AccountMeta::new(history_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };

        let recent_blockhash = self.get_latest_blockhash_with_retry(5, Duration::from_secs(5)).await?;
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
        transaction.message.recent_blockhash = recent_blockhash;
        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);

        let serialized_transaction = utils::serialize_transaction(&transaction)?;
        let metadata = serde_json::to_string(req)?;

        Ok(PreparedTransaction {
            serialized_transaction,
            transaction_type: "add_read_authority".to_string(),
            metadata,
        })
    }

    pub async fn prepare_remove_read_authority(&self, req: &RemoveReadAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        log::info!("Preparing remove_read_authority transaction for authority: {}", req.authority_to_remove);

        let user_pubkey = Pubkey::from_str(&req.user_pubkey)
            .map_err(|e| AppError::BadRequest(format!("Invalid user public key: {}", e)))?;

        let authority_to_remove = Pubkey::from_str(&req.authority_to_remove)
            .map_err(|e| AppError::BadRequest(format!("Invalid authority to remove public key: {}", e)))?;

        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );

        let mut instruction_data = vec![184, 21, 123, 83, 88, 34, 159, 122];
        instruction_data.extend_from_slice(authority_to_remove.as_ref());

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin_pubkey, true),
                AccountMeta::new(admin_pda, false),
                AccountMeta::new(history_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };

        let recent_blockhash = self.get_latest_blockhash_with_retry(5, Duration::from_secs(5)).await?;
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
        transaction.message.recent_blockhash = recent_blockhash;
        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);

        let serialized_transaction = utils::serialize_transaction(&transaction)?;
        let metadata = serde_json::to_string(req)?;

        Ok(PreparedTransaction {
            serialized_transaction,
            transaction_type: "remove_read_authority".to_string(),
            metadata,
        })
    }

    pub async fn prepare_add_write_authority(&self, req: &AddWriteAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        log::info!("Preparing add_write_authority transaction for new authority: {}", req.new_authority);

        let user_pubkey = Pubkey::from_str(&req.user_pubkey)
            .map_err(|e| AppError::BadRequest(format!("Invalid user public key: {}", e)))?;

        let new_authority = Pubkey::from_str(&req.new_authority)
            .map_err(|e| AppError::BadRequest(format!("Invalid new authority public key: {}", e)))?;

        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );

        let mut instruction_data = vec![82, 195, 138, 26, 4, 176, 126, 226];
        instruction_data.extend_from_slice(new_authority.as_ref());

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin_pubkey, true),
                AccountMeta::new(admin_pda, false),
                AccountMeta::new(history_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };

        let recent_blockhash = self.get_latest_blockhash_with_retry(5, Duration::from_secs(5)).await?;
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
        transaction.message.recent_blockhash = recent_blockhash;
        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);

        let serialized_transaction = utils::serialize_transaction(&transaction)?;
        let metadata = serde_json::to_string(req)?;

        Ok(PreparedTransaction {
            serialized_transaction,
            transaction_type: "add_write_authority".to_string(),
            metadata,
        })
    }

    pub async fn prepare_remove_write_authority(&self, req: &RemoveWriteAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        log::info!("Preparing remove_write_authority transaction for authority: {}", req.authority_to_remove);

        let user_pubkey = Pubkey::from_str(&req.user_pubkey)
            .map_err(|e| AppError::BadRequest(format!("Invalid user public key: {}", e)))?;

        let authority_to_remove = Pubkey::from_str(&req.authority_to_remove)
            .map_err(|e| AppError::BadRequest(format!("Invalid authority to remove public key: {}", e)))?;

        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );

        let mut instruction_data = vec![60, 67, 110, 202, 138, 63, 172, 59];
        instruction_data.extend_from_slice(authority_to_remove.as_ref());

        let instruction = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.admin_pubkey, true),
                AccountMeta::new(admin_pda, false),
                AccountMeta::new(history_pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        };

        let recent_blockhash = self.get_latest_blockhash_with_retry(5, Duration::from_secs(5)).await?;
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&user_pubkey));
        transaction.message.recent_blockhash = recent_blockhash;
        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);

        let serialized_transaction = utils::serialize_transaction(&transaction)?;
        let metadata = serde_json::to_string(req)?;

        Ok(PreparedTransaction {
            serialized_transaction,
            transaction_type: "remove_write_authority".to_string(),
            metadata,
        })
    }

    pub async fn submit_transaction(&self, serialized_transaction: &str) -> Result<String, AppError> {
        log::info!("Submitting transaction: {}", serialized_transaction);

        let transaction_bytes = STANDARD
            .decode(serialized_transaction)
            .map_err(|e| AppError::BadRequest(format!("Failed to decode transaction: {}", e)))?;

        let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)
            .map_err(|e| AppError::BadRequest(format!("Failed to deserialize transaction: {}", e)))?;

        let recent_blockhash = self.get_latest_blockhash_with_retry(5, Duration::from_secs(5)).await?;
        transaction.message.recent_blockhash = recent_blockhash;

        let user_pubkey = transaction.message.account_keys[0];
        let mut user_signature = None;
        for (i, sig) in transaction.signatures.iter().enumerate() {
            let pubkey = transaction.message.account_keys[i];
            if pubkey == user_pubkey && sig != &solana_sdk::signature::Signature::default() {
                user_signature = Some(*sig);
                break;
            }
        }

        transaction.signatures = transaction
            .message
            .account_keys
            .iter()
            .map(|pubkey| {
                if *pubkey == user_pubkey {
                    solana_sdk::signature::Signature::default()
                } else {
                    solana_sdk::signature::Signature::default()
                }
            })
            .collect();

        transaction.partial_sign(&[&self.admin_keypair], recent_blockhash);

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

        let signature = self
            .client
            .send_and_confirm_transaction(&transaction)
            .await
            .map_err(|e| AppError::SolanaError(format!("Failed to submit transaction: {}", e)))?;

        log::info!("Transaction submitted with signature: {}", signature);
        Ok(signature.to_string())
    }

    pub async fn get_authorities(&self) -> Result<AuthoritiesResponse, AppError> {
        log::info!("Fetching authorities list");

        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);

        let account_info = self.client
            .get_account(&admin_pda)
            .await
            .map_err(|e| AppError::SolanaError(format!("Failed to fetch admin account: {}", e)))?;

        if account_info.owner != self.program_id {
            return Err(AppError::BadRequest("Admin account not owned by program".to_string()));
        }

        #[derive(BorshDeserialize)]
        struct AdminAccount {
            authority: Pubkey,
            read_authorities: Vec<Pubkey>,
            write_authorities: Vec<Pubkey>,
        }

        let data = &account_info.data[8..];
        let admin: AdminAccount = BorshDeserialize::deserialize(&mut data.as_ref())
            .map_err(|e| AppError::SolanaError(format!("Failed to deserialize admin account: {}", e)))?;

        let response = AuthoritiesResponse {
            authority: admin.authority.to_string(),
            read_authorities: admin.read_authorities.into_iter().map(|pk| pk.to_string()).collect(),
            write_authorities: admin.write_authorities.into_iter().map(|pk| pk.to_string()).collect(),
        };

        log::info!("Authorities fetched: {:?}", response);
        Ok(response)
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