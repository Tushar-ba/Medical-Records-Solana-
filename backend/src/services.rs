use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use solana_client::rpc_filter::{RpcFilterType, Memcmp}; // Remove duplicate import
use solana_account_decoder::UiAccountEncoding;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use bincode;
use borsh::BorshDeserialize;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{
    AddReadAuthorityRequest, RemoveReadAuthorityRequest, AddWriteAuthorityRequest,
    RemoveWriteAuthorityRequest, PreparedTransaction, CreatePatientRequest,
    PreparedPatientTransaction, UpdatePatientRequest, PreparedUpdatePatientTransaction,
    GetPatientResponse, PatientAddressesResponse, AuthorityHistoryResponse, HistoryEntry,
};
use crate::utils;
use rand::Rng;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use uuid::Uuid;
use solana_client::rpc_config::{RpcProgramAccountsConfig, RpcAccountInfoConfig};

pub struct TransactionService {
    client: RpcClient,
    admin_keypair: Keypair,
    admin_pubkey: Pubkey,
    program_id: Pubkey,
    encryption_key: [u8; 32],
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
        let user_pubkey = Pubkey::from_str(&req.user_pubkey)?;
        let patient_seed = Keypair::new();
        let patient_seed_pubkey = patient_seed.pubkey();
        let (patient_pda, _bump) = Pubkey::find_program_address(
            &[b"patient", user_pubkey.as_ref(), patient_seed_pubkey.as_ref()],
            &self.program_id,
        );
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let nonce_bytes = rand::thread_rng().gen::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let encrypted_data = cipher.encrypt(nonce, req.patient_data.as_bytes())
            .map_err(|e| AppError::InternalServerError(format!("Encryption failed: {}", e)))?;
        let encrypted_data_base64 = STANDARD.encode(&encrypted_data);
        let nonce_base64 = STANDARD.encode(&nonce_bytes);
        let encrypted_string = format!("{}|{}", encrypted_data_base64, nonce_base64);
        let discriminator = [176, 85, 210, 156, 179, 74, 60, 203];
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

    pub async fn prepare_update_patient(&self, req: &UpdatePatientRequest) -> Result<PreparedUpdatePatientTransaction, AppError> {
        log::info!("Preparing update_patient transaction for user: {}", req.user_pubkey);
        let user_pubkey = Pubkey::from_str(&req.user_pubkey)?;
        let patient_seed_pubkey = Pubkey::from_str(&req.patient_seed)?;
        let (patient_pda, _bump) = Pubkey::find_program_address(
            &[b"patient", user_pubkey.as_ref(), patient_seed_pubkey.as_ref()],
            &self.program_id,
        );
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let nonce_bytes = rand::thread_rng().gen::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let encrypted_data = cipher.encrypt(nonce, req.patient_data.as_bytes())
            .map_err(|e| AppError::InternalServerError(format!("Encryption failed: {}", e)))?;
        let encrypted_data_base64 = STANDARD.encode(&encrypted_data);
        let nonce_base64 = STANDARD.encode(&nonce_bytes);
        let encrypted_string = format!("{}|{}", encrypted_data_base64, nonce_base64);
        let discriminator = [112, 151, 255, 60, 59, 88, 232, 154];
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
        Ok(PreparedUpdatePatientTransaction {
            serialized_transaction,
            transaction_type: "update_patient".to_string(),
            encrypted_data_with_seed,
        })
    }

    pub async fn prepare_add_read_authority(&self, req: &AddReadAuthorityRequest) -> Result<PreparedTransaction, AppError> {
        log::info!("Preparing add_read_authority transaction for new authority: {}", req.new_authority);
        let user_pubkey = Pubkey::from_str(&req.user_pubkey)?;
        let new_authority = Pubkey::from_str(&req.new_authority)?;
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(&[b"history", self.admin_pubkey.as_ref()], &self.program_id);
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
        let user_pubkey = Pubkey::from_str(&req.user_pubkey)?;
        let authority_to_remove = Pubkey::from_str(&req.authority_to_remove)?;
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(&[b"history", self.admin_pubkey.as_ref()], &self.program_id);
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
        let user_pubkey = Pubkey::from_str(&req.user_pubkey)?;
        let new_authority = Pubkey::from_str(&req.new_authority)?;
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(&[b"history", self.admin_pubkey.as_ref()], &self.program_id);
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
        let user_pubkey = Pubkey::from_str(&req.user_pubkey)?;
        let authority_to_remove = Pubkey::from_str(&req.authority_to_remove)?;
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let (history_pda, _bump) = Pubkey::find_program_address(&[b"history", self.admin_pubkey.as_ref()], &self.program_id);
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
        let transaction_bytes = STANDARD.decode(serialized_transaction)?;
        let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)?;
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
        transaction.signatures = transaction.message.account_keys.iter().map(|pubkey| {
            if *pubkey == user_pubkey {
                solana_sdk::signature::Signature::default()
            } else {
                solana_sdk::signature::Signature::default()
            }
        }).collect();
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
        let signature = self.client.send_and_confirm_transaction(&transaction).await?;
        log::info!("Transaction submitted with signature: {}", signature);
        Ok(signature.to_string())
    }

    pub async fn get_authorities(&self) -> Result<crate::models::AuthoritiesResponse, AppError> {
        log::info!("Fetching authorities list");
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let account_info = self.client.get_account(&admin_pda).await?;
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
        let admin: AdminAccount = BorshDeserialize::deserialize(&mut data.as_ref())?;
        let response = crate::models::AuthoritiesResponse {
            authority: admin.authority.to_string(),
            read_authorities: admin.read_authorities.into_iter().map(|pk| pk.to_string()).collect(),
            write_authorities: admin.write_authorities.into_iter().map(|pk| pk.to_string()).collect(),
        };
        log::info!("Authorities fetched: {:?}", response);
        Ok(response)
    }

    pub async fn get_patient(&self, patient_seed: &str, user_pubkey: &str, app_state: &AppState) -> Result<GetPatientResponse, AppError> {
        log::info!("Fetching patient data for seed: {}", patient_seed);

        let user_pubkey = Pubkey::from_str(user_pubkey)?;
        let patient_seed_pubkey = Pubkey::from_str(patient_seed)?;
        let (patient_pda, _bump) = Pubkey::find_program_address(
            &[b"patient", user_pubkey.as_ref(), patient_seed_pubkey.as_ref()],
            &self.program_id,
        );
        let (_admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);

        let account_info = self.client.get_account(&patient_pda).await?;
        if account_info.owner != self.program_id {
            return Err(AppError::BadRequest("Patient account not owned by program".to_string()));
        }

        #[derive(BorshDeserialize)]
        struct PatientAccount {
            patient_address: Pubkey,
            is_initialized: bool,
            encrypted_data: String,
            data_hash: [u8; 32],
        }

        let data = &account_info.data[8..];
        let patient: PatientAccount = BorshDeserialize::deserialize(&mut data.as_ref())?;

        if !patient.is_initialized {
            return Err(AppError::BadRequest("Patient record not initialized".to_string()));
        }

        let token = Uuid::new_v4().to_string();
        let expiration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600;
        app_state.token_store.insert(token.clone(), (patient_seed.to_string(), expiration));

        let view_url = format!("http://localhost:8080/api/view_patient/{}", token);

        Ok(GetPatientResponse { view_url })
    }

    pub async fn view_patient(&self, token: &str, app_state: &AppState) -> Result<String, AppError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = app_state.token_store.get(token).ok_or_else(|| {
            AppError::Unauthorized("Invalid or expired token".to_string())
        })?;

        let (patient_seed, expiration) = entry.value();
        if now > *expiration {
            app_state.token_store.remove(token);
            return Err(AppError::Unauthorized("Token expired".to_string()));
        }

        let patient_seed_pubkey = Pubkey::from_str(patient_seed)?;
        let (admin_pda, _bump) = Pubkey::find_program_address(&[b"admin"], &self.program_id);
        let account_info = self.client.get_account(&admin_pda).await?;
        let admin_data = &account_info.data[8..];
        #[derive(BorshDeserialize)]
        struct AdminAccount {
            authority: Pubkey,
            read_authorities: Vec<Pubkey>,
            write_authorities: Vec<Pubkey>,
        }
        let admin: AdminAccount = BorshDeserialize::deserialize(&mut admin_data.as_ref())?;
        let (patient_pda, _bump) = Pubkey::find_program_address(
            &[b"patient", admin.authority.as_ref(), patient_seed_pubkey.as_ref()],
            &self.program_id,
        );

        let patient_account_info = self.client.get_account(&patient_pda).await?;
        let patient_data = &patient_account_info.data[8..];
        #[derive(BorshDeserialize)]
        struct PatientAccount {
            patient_address: Pubkey,
            is_initialized: bool,
            encrypted_data: String,
            data_hash: [u8; 32],
        }
        let patient: PatientAccount = BorshDeserialize::deserialize(&mut patient_data.as_ref())?;

        let parts: Vec<&str> = patient.encrypted_data.split('|').collect();
        if parts.len() != 2 {
            return Err(AppError::InternalServerError("Invalid encrypted data format".to_string()));
        }
        let encrypted_data = STANDARD.decode(parts[0])?;
        let nonce = STANDARD.decode(parts[1])?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let nonce = Nonce::from_slice(&nonce);
        let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| AppError::InternalServerError(format!("Decryption failed: {}", e)))?;

        Ok(String::from_utf8(decrypted_data).unwrap_or_else(|_| "Decrypted data not UTF-8".to_string()))
    }

    pub async fn get_authority_history(&self) -> Result<AuthorityHistoryResponse, AppError> {
        log::info!("Fetching authority history");
        let (history_pda, _bump) = Pubkey::find_program_address(
            &[b"history", self.admin_pubkey.as_ref()],
            &self.program_id,
        );

        let account_info = self.client.get_account(&history_pda).await?;
        if account_info.owner != self.program_id {
            return Err(AppError::BadRequest(
                "History account not owned by program".to_string(),
            ));
        }

        #[derive(BorshDeserialize)]
        struct LocalHistoryEntry {
            admin: Pubkey,
            authority: Pubkey,
            added: bool,
            is_read: bool,
            timestamp: i64,
        }

        #[derive(BorshDeserialize)]
        struct AuthorityHistory {
            entries: Vec<LocalHistoryEntry>,
        }

        let data = &account_info.data[8..];
        let history: AuthorityHistory = BorshDeserialize::deserialize(&mut data.as_ref())?;

        let response = AuthorityHistoryResponse {
            entries: history
                .entries
                .into_iter()
                .map(|entry| HistoryEntry {
                    admin: entry.admin.to_string(),
                    authority: entry.authority.to_string(),
                    added: entry.added,
                    is_read: entry.is_read,
                    timestamp: entry.timestamp,
                })
                .collect(),
        };

        log::info!("Authority history fetched with {} entries", response.entries.len());
        Ok(response)
    }

    pub async fn get_patient_addresses(&self, _user_pubkey: &str) -> Result<PatientAddressesResponse, AppError> {
        log::info!("Fetching all patient addresses for user: {}", _user_pubkey);
        let config = RpcProgramAccountsConfig {
            filters: Some(vec![
                RpcFilterType::Memcmp(
                    Memcmp::new(
                        0, // offset
                        solana_client::rpc_filter::MemcmpEncodedBytes::Base58(
                            bs58::encode(&[118, 127, 39, 235, 201, 189, 0, 109]).into_string()
                        ),
                    )
                ),
            ]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            },
            ..Default::default()
        };
    
        let accounts = self.client
            .get_program_accounts_with_config(&self.program_id, config)
            .await?;
    
        let patient_addresses = accounts
            .into_iter()
            .map(|(pubkey, _account)| pubkey.to_string())
            .collect();
    
        let response = PatientAddressesResponse {
            patient_addresses,
        };
    
        log::info!("Fetched {} patient addresses", response.patient_addresses.len());
        Ok(response)
    }

    async fn get_latest_blockhash_with_retry(
        &self,
        retries: u32,
        delay: Duration,
    ) -> Result<solana_sdk::hash::Hash, AppError> {
        for attempt in 1..=retries {
            match self.client.get_latest_blockhash().await {
                Ok(blockhash) => return Ok(blockhash),
                Err(e) => {
                    log::error!("Attempt {}/{}: Failed to get latest blockhash: {}", attempt, retries, e);
                    if attempt == retries {
                        return Err(AppError::SolanaError(format!("Failed to get latest blockhash after {} attempts: {}", retries, e)));
                    }
                    sleep(Duration::from_millis(delay.as_millis() as u64)).await;
                }
            }
        }
        Err(AppError::SolanaError("Failed to get latest blockhash".to_string()))
    }
}