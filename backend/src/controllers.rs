use actix_web::{web, HttpResponse, Responder};
use jsonwebtoken::{encode, EncodingKey, Header};
use solana_sdk::signature::Signature;
use bs58;
use chrono;
use log::{info, error};

use crate::{
    models::{AuthRequest, AuthResponse, AddReadAuthorityRequest, SubmitTransactionRequest, SubmitTransactionResponse},
    app_state::AppState,
    error::AppError,
};

pub async fn authenticate(
    req: web::Json<AuthRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Received authentication request for public key: {}", req.public_key);

    let config = data.config.lock().unwrap();
    
    let signature_bytes = bs58::decode(&req.signature)
        .into_vec()
        .map_err(|e| AppError::BadRequest(format!("Failed to decode signature: {}", e)))?;
    
    let pubkey_bytes = bs58::decode(&req.public_key)
        .into_vec()
        .map_err(|e| AppError::BadRequest(format!("Failed to decode public key: {}", e)))?;

    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|e| AppError::BadRequest(format!("Invalid signature bytes: {}", e)))?;

    let message = format!("Timestamp: {}", req.timestamp);
    let message_bytes = message.as_bytes();
    let verified = signature.verify(&pubkey_bytes, message_bytes);

    if !verified {
        error!("Signature verification failed for public key: {}", req.public_key);
        return Err(AppError::Unauthorized("Signature verification failed".to_string()));
    }

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(config.token_expiration))
        .expect("valid timestamp")
        .timestamp();

    let claims = crate::models::Claims {
        sub: req.public_key.clone(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to create JWT: {}", e)))?;

    info!("Successfully authenticated user with public key: {}. Token generated.", req.public_key);
    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        expires_in: config.token_expiration,
        public_key: req.public_key.clone(),
    }))
}

pub async fn prepare_add_read_authority(
    req: web::Json<AddReadAuthorityRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Received prepare_add_read_authority request for new authority: {}", req.new_authority);
    let prepared_tx = data.solana_service.prepare_add_read_authority(&req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn submit_transaction(
    req: web::Json<SubmitTransactionRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Received request to submit transaction");
    let signature = data.solana_service.submit_transaction(&req.serialized_transaction).await?;
    Ok(HttpResponse::Ok().json(SubmitTransactionResponse { signature }))
}