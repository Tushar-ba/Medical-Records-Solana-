use actix_web::{web, HttpResponse, Responder};
use jsonwebtoken::{encode, EncodingKey, Header};
use solana_sdk::signature::Signature;
use bs58;
use chrono;
use log::{info, error};

use crate::{models::{AuthRequest, AuthResponse, Claims}, app_state::AppState};

pub async fn authenticate(
    req: web::Json<AuthRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    info!("Received authentication request for public key: {}", req.public_key);

    let config = data.config.lock().unwrap();
    
    let signature_bytes = match bs58::decode(&req.signature).into_vec() {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode signature: {}", e);
            return HttpResponse::BadRequest().json("Invalid signature format");
        }
    };
    
    let pubkey_bytes = match bs58::decode(&req.public_key).into_vec() {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode public key: {}", e);
            return HttpResponse::BadRequest().json("Invalid public key format");
        }
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(e) => {
            error!("Invalid signature bytes: {}", e);
            return HttpResponse::BadRequest().json("Invalid signature");
        }
    };

    let message_bytes = req.message.as_bytes();
    let verified = signature.verify(&pubkey_bytes, message_bytes);

    if !verified {
        error!("Signature verification failed for public key: {}", req.public_key);
        return HttpResponse::Unauthorized().json("Signature verification failed");
    }

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(config.token_expiration))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: req.public_key.clone(),
        exp: expiration,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create JWT: {}", e);
            return HttpResponse::InternalServerError().json("Token creation failed");
        }
    };

    info!("Successfully authenticated user with public key: {}. Token generated.", req.public_key);
    HttpResponse::Ok().json(AuthResponse { token })
}