use actix_web::{web, HttpResponse};
use log::{info, error};
use solana_sdk::signature::Signature;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr; // Added import
use bs58;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::jwt::generate_jwt;

pub async fn authenticate(
    req: web::Json<crate::models::AuthRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    info!("Received authentication request for public key: {}", req.public_key);
    let signature_bytes = bs58::decode(&req.signature).into_vec()?;
    let pubkey_bytes = bs58::decode(&req.public_key).into_vec()?;
    let signature = Signature::try_from(signature_bytes.as_slice())?;
    let message = format!("Timestamp: {}", req.timestamp);
    let message_bytes = message.as_bytes();
    let verified = signature.verify(&pubkey_bytes, message_bytes);
    if !verified {
        error!("Signature verification failed for public key: {}", req.public_key);
        return Err(AppError::Unauthorized("Signature verification failed".to_string()));
    }
    let token = generate_jwt(&req.public_key, &data.jwt_config.secret, data.jwt_config.expires_in)?;
    let response = crate::models::AuthResponse {
        token,
        expires_in: data.jwt_config.expires_in,
        public_key: req.public_key.clone(),
    };
    info!("Successfully authenticated user with public key: {}. Token generated.", req.public_key);
    Ok(HttpResponse::Ok().json(response))
}

pub async fn prepare_add_read_authority(
    req: web::Json<crate::models::AddReadAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_add_read_authority request for new authority: {}", req.new_authority);
    let user_pubkey = req_data.into_inner();
    let modified_req = crate::models::AddReadAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        new_authority: req.new_authority.clone(),
    };
    let prepared_tx = data.solana_service.prepare_add_read_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_remove_read_authority(
    req: web::Json<crate::models::RemoveReadAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_remove_read_authority request for authority: {}", req.authority_to_remove);
    let user_pubkey = req_data.into_inner();
    let modified_req = crate::models::RemoveReadAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        authority_to_remove: req.authority_to_remove.clone(),
    };
    let prepared_tx = data.solana_service.prepare_remove_read_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_add_write_authority(
    req: web::Json<crate::models::AddWriteAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_add_write_authority request for new authority: {}", req.new_authority);
    let user_pubkey = req_data.into_inner();
    let modified_req = crate::models::AddWriteAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        new_authority: req.new_authority.clone(),
    };
    let prepared_tx = data.solana_service.prepare_add_write_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_remove_write_authority(
    req: web::Json<crate::models::RemoveWriteAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_remove_write_authority request for authority: {}", req.authority_to_remove);
    let user_pubkey = req_data.into_inner();
    let modified_req = crate::models::RemoveWriteAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        authority_to_remove: req.authority_to_remove.clone(),
    };
    let prepared_tx = data.solana_service.prepare_remove_write_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_create_patient(
    req: web::Json<crate::models::CreatePatientRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_create_patient request");
    let user_pubkey = req_data.into_inner();
    let modified_req = crate::models::CreatePatientRequest {
        user_pubkey: user_pubkey.clone(),
        patient_data: req.patient_data.clone(),
    };
    let prepared_tx = data.solana_service.prepare_create_patient(&modified_req).await?;
    
    // Extract patient_seed and patient_pda to store in AppState
    let parts: Vec<&str> = prepared_tx.encrypted_data_with_seed.split('|').collect();
    if parts.len() >= 3 {
        let patient_seed = parts[2].to_string();
        let patient_pda = Pubkey::find_program_address(
            &[b"patient", data.solana_service.admin_pubkey.as_ref(), Pubkey::from_str(&patient_seed)?.as_ref()],
            &data.solana_service.program_id,
        ).0.to_string();
        data.patient_seed_map.insert(patient_pda, patient_seed);
    }
    
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_update_patient(
    req: web::Json<crate::models::UpdatePatientRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_update_patient request");
    let user_pubkey = req_data.into_inner();
    let modified_req = crate::models::UpdatePatientRequest {
        user_pubkey,
        patient_seed: req.patient_seed.clone(),
        patient_data: req.patient_data.clone(),
    };
    let prepared_tx = data.solana_service.prepare_update_patient(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn submit_transaction(
    req: web::Json<crate::models::SubmitTransactionRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    info!("Received submit_transaction request");
    let signature = data.solana_service.submit_transaction(&req.serialized_transaction).await?;
    let response = crate::models::SubmitTransactionResponse { signature };
    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_authorities(
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    info!("Received get_authorities request");
    let authorities = data.solana_service.get_authorities().await?;
    Ok(HttpResponse::Ok().json(authorities))
}

pub async fn get_patient(
    path: web::Path<String>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    let patient_seed = path.into_inner();
    let user_pubkey = req_data.into_inner();
    info!("Received get_patient request for seed: {}", patient_seed);
    let response = data.solana_service.get_patient(&patient_seed, &user_pubkey, &data).await?;
    Ok(HttpResponse::Ok().json(response))
}

pub async fn view_patient(
    path: web::Path<String>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    let token = path.into_inner();
    let _user_pubkey = req_data.into_inner();
    info!("Received view_patient request for token: {}", token);
    let decrypted_data = data.solana_service.view_patient(&token, &data).await?;
    Ok(HttpResponse::Ok().body(decrypted_data))
}

pub async fn get_authority_history(
    data: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    info!("Received get_authority_history request");
    let history = data.solana_service.get_authority_history().await?;
    Ok(HttpResponse::Ok().json(history))
}

pub async fn get_patient_addresses(
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    let user_pubkey = req_data.into_inner();
    info!("Received get_patient_addresses request for user: {}", user_pubkey);
    let addresses = data.solana_service.get_patient_addresses(&user_pubkey, &data).await?;
    Ok(HttpResponse::Ok().json(addresses))
}