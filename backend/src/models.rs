use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expires_in: i64,
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
    pub user_pubkey: String,
    pub new_authority: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveReadAuthorityRequest {
    pub user_pubkey: String,
    pub authority_to_remove: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddWriteAuthorityRequest {
    pub user_pubkey: String,
    pub new_authority: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveWriteAuthorityRequest {
    pub user_pubkey: String,
    pub authority_to_remove: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePatientRequest {
    pub user_pubkey: String,
    pub patient_data: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdatePatientRequest {
    pub user_pubkey: String,
    pub patient_seed: String,
    pub patient_data: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub serialized_transaction: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct PreparedTransaction {
    pub serialized_transaction: String,
    pub transaction_type: String,
    pub metadata: String,
}

#[derive(Serialize, Deserialize)]
pub struct PreparedPatientTransaction {
    pub serialized_transaction: String,
    pub transaction_type: String,
    pub encrypted_data_with_seed: String,
}

#[derive(Serialize, Deserialize)]
pub struct PreparedUpdatePatientTransaction {
    pub serialized_transaction: String,
    pub transaction_type: String,
    pub encrypted_data_with_seed: String,
}

#[derive(Serialize, Deserialize, Debug)] // Added Debug
pub struct AuthoritiesResponse {
    pub authority: String,
    pub read_authorities: Vec<String>,
    pub write_authorities: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetPatientResponse {
    pub view_url: String,
}