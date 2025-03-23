use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Wallet public key
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AuthRequest {
    pub public_key: String,
    pub signature: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: i64,
    pub public_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddReadAuthorityRequest {
    pub new_authority: String, // Public key of the new read authority
}

#[derive(Serialize, Deserialize)]
pub struct PreparedTransaction {
    pub serialized_transaction: String,
    pub transaction_type: String,
    pub metadata: String, // JSON string of the request data
}

#[derive(Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub signature: String,
    pub serialized_transaction: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub signature: String,
}