use solana_sdk::transaction::Transaction;
use base64::{engine::general_purpose, Engine as _};
use bincode;
use crate::error::AppError;

pub fn serialize_transaction(transaction: &Transaction) -> Result<String, AppError> {
    let serialized = bincode::serialize(transaction)
        .map_err(|e| AppError::InternalServerError(format!("Failed to serialize transaction: {}", e)))?;
    Ok(general_purpose::STANDARD.encode(serialized))
}

pub fn deserialize_transaction(serialized: &str) -> Result<Transaction, AppError> {
    let transaction_bytes = general_purpose::STANDARD.decode(serialized)
        .map_err(|e| AppError::BadRequest(format!("Failed to decode transaction: {}", e)))?;
    let transaction = bincode::deserialize(&transaction_bytes)
        .map_err(|e| AppError::BadRequest(format!("Failed to deserialize transaction: {}", e)))?;
    Ok(transaction)
}