use actix_web::{web, HttpResponse};
use log::{info, error};
use solana_sdk::signature::Signature;
use bs58;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::jwt::generate_jwt;
use crate::models::{AuthRequest, AuthResponse, AddReadAuthorityRequest, SubmitTransactionRequest, SubmitTransactionResponse};

pub async fn authenticate(
    req: web::Json<AuthRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    info!("Received authentication request for public key: {}", req.public_key);

    // Decode the signature and public key
    let signature_bytes = bs58::decode(&req.signature)
        .into_vec()
        .map_err(|e| AppError::BadRequest(format!("Failed to decode signature: {}", e)))?;
    
    let pubkey_bytes = bs58::decode(&req.public_key)
        .into_vec()
        .map_err(|e| AppError::BadRequest(format!("Failed to decode public key: {}", e)))?;

    // Verify the signature
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|e| AppError::BadRequest(format!("Invalid signature bytes: {}", e)))?;

    let message = format!("Timestamp: {}", req.timestamp);
    let message_bytes = message.as_bytes();
    let verified = signature.verify(&pubkey_bytes, message_bytes);

    if !verified {
        error!("Signature verification failed for public key: {}", req.public_key);
        return Err(AppError::Unauthorized("Signature verification failed".to_string()));
    }

    // Generate JWT using the middleware function
    let token = generate_jwt(&req.public_key, &data.jwt_config.secret, data.jwt_config.expires_in)?;
    let response = AuthResponse {
        token,
        expires_in: data.jwt_config.expires_in,
        public_key: req.public_key.clone(),
    };

    info!("Successfully authenticated user with public key: {}. Token generated.", req.public_key);
    Ok(HttpResponse::Ok().json(response))
}

pub async fn prepare_add_read_authority(
    req: web::Json<AddReadAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_add_read_authority request for new authority: {}", req.new_authority);

    // Extract the user's public key from the JWT token
    let user_pubkey = req_data.into_inner();

    // Create a modified request with the user_pubkey
    let modified_req = AddReadAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        new_authority: req.new_authority.clone(),
    };

    let prepared_tx = data.solana_service.prepare_add_read_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn submit_transaction(
    req: web::Json<SubmitTransactionRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    info!("Received submit_transaction request");
    let signature = data.solana_service.submit_transaction(&req.serialized_transaction).await?;
    let response = SubmitTransactionResponse { signature };
    Ok(HttpResponse::Ok().json(response))
}