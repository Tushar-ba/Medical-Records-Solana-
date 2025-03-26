use actix_web::{web, HttpResponse};
use log::{info, error};
use solana_sdk::signature::Signature;
use bs58;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::jwt::generate_jwt;
use crate::models::{AuthRequest, AuthResponse, AddReadAuthorityRequest, RemoveReadAuthorityRequest, AddWriteAuthorityRequest, RemoveWriteAuthorityRequest, SubmitTransactionRequest, SubmitTransactionResponse, CreatePatientRequest, PreparedPatientTransaction, UpdatePatientRequest, PreparedUpdatePatientTransaction, GetPatientResponse};

pub async fn authenticate(
    req: web::Json<AuthRequest>,
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
    let user_pubkey = req_data.into_inner();
    let modified_req = AddReadAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        new_authority: req.new_authority.clone(),
    };
    let prepared_tx = data.solana_service.prepare_add_read_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_remove_read_authority(
    req: web::Json<RemoveReadAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_remove_read_authority request for authority: {}", req.authority_to_remove);
    let user_pubkey = req_data.into_inner();
    let modified_req = RemoveReadAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        authority_to_remove: req.authority_to_remove.clone(),
    };
    let prepared_tx = data.solana_service.prepare_remove_read_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_add_write_authority(
    req: web::Json<AddWriteAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_add_write_authority request for new authority: {}", req.new_authority);
    let user_pubkey = req_data.into_inner();
    let modified_req = AddWriteAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        new_authority: req.new_authority.clone(),
    };
    let prepared_tx = data.solana_service.prepare_add_write_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_remove_write_authority(
    req: web::Json<RemoveWriteAuthorityRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_remove_write_authority request for authority: {}", req.authority_to_remove);
    let user_pubkey = req_data.into_inner();
    let modified_req = RemoveWriteAuthorityRequest {
        user_pubkey: user_pubkey.clone(),
        authority_to_remove: req.authority_to_remove.clone(),
    };
    let prepared_tx = data.solana_service.prepare_remove_write_authority(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_create_patient(
    req: web::Json<CreatePatientRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_create_patient request");
    let user_pubkey = req_data.into_inner();
    let modified_req = CreatePatientRequest {
        user_pubkey,
        patient_data: req.patient_data.clone(),
    };
    let prepared_tx = data.solana_service.prepare_create_patient(&modified_req).await?;
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_update_patient(
    req: web::Json<UpdatePatientRequest>,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_update_patient request");
    let user_pubkey = req_data.into_inner();
    let modified_req = UpdatePatientRequest {
        user_pubkey,
        patient_seed: req.patient_seed.clone(),
        patient_data: req.patient_data.clone(),
    };
    let prepared_tx = data.solana_service.prepare_update_patient(&modified_req).await?;
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
    let _user_pubkey = req_data.into_inner(); // JWT already verified
    info!("Received view_patient request for token: {}", token);
    let decrypted_data = data.solana_service.view_patient(&token, &data).await?;
    Ok(HttpResponse::Ok().body(decrypted_data))
}