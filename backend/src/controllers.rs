use actix_web::{web, HttpResponse};
use actix_multipart::Multipart;
use futures_util::TryStreamExt as _;
use log::{info, error};
use solana_sdk::signature::Signature;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
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
    mut payload: Multipart,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_create_patient request");
    let user_pubkey_from_token = req_data.into_inner();

    let mut user_pubkey = String::new();
    let mut name = String::new();
    let mut blood_type = String::new();
    let mut previous_report = String::new();
    let mut ph_no = String::new();
    let mut file_data = Vec::new();

    while let Some(mut field) = payload.try_next().await? {
        match field.name() {
            "user_pubkey" => {
                while let Some(chunk) = field.try_next().await? {
                    user_pubkey.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "name" => {
                while let Some(chunk) = field.try_next().await? {
                    name.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "blood_type" => {
                while let Some(chunk) = field.try_next().await? {
                    blood_type.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "previous_report" => {
                while let Some(chunk) = field.try_next().await? {
                    previous_report.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "ph_no" => {
                while let Some(chunk) = field.try_next().await? {
                    ph_no.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "file" => {
                while let Some(chunk) = field.try_next().await? {
                    file_data.extend_from_slice(chunk.as_ref());
                }
            }
            _ => {}
        }
    }

    // Validate inputs
    if user_pubkey.is_empty() {
        user_pubkey = user_pubkey_from_token;
    } else if user_pubkey != user_pubkey_from_token {
        error!("User pubkey from form ({}) does not match JWT ({})", user_pubkey, user_pubkey_from_token);
        return Err(AppError::Unauthorized("User pubkey mismatch".to_string()));
    }
    if name.is_empty() || blood_type.is_empty() || previous_report.is_empty() || ph_no.is_empty() {
        error!("Missing required patient data fields");
        return Err(AppError::BadRequest("Missing required patient data fields".to_string()));
    }

    let patient_data = crate::models::PatientData {
        name,
        blood_type,
        previous_report,
        ph_no,
        file: None, // Will be updated with CID if file is uploaded
    };

    let modified_req = crate::models::CreatePatientRequest {
        user_pubkey: user_pubkey.clone(),
        patient_data,
    };

    // Pass AppState to prepare_create_patient
    let prepared_tx = if file_data.is_empty() {
        data.solana_service.prepare_create_patient(&modified_req, None, &data).await?
    } else {
        data.solana_service.prepare_create_patient(&modified_req, Some(&file_data), &data).await?
    };

    // Remove the redundant patient_seed_map insertion here since it's now handled in prepare_create_patient
    Ok(HttpResponse::Ok().json(prepared_tx))
}

pub async fn prepare_update_patient(
    mut payload: Multipart,
    data: web::Data<AppState>,
    req_data: web::ReqData<String>,
) -> Result<HttpResponse, AppError> {
    info!("Received prepare_update_patient request");
    let user_pubkey_from_token = req_data.into_inner();

    let mut user_pubkey = String::new();
    let mut patient_seed = String::new();
    let mut name = String::new();
    let mut blood_type = String::new();
    let mut previous_report = String::new();
    let mut ph_no = String::new();
    let mut file_data = Vec::new();

    while let Some(mut field) = payload.try_next().await? {
        match field.name() {
            "user_pubkey" => {
                while let Some(chunk) = field.try_next().await? {
                    user_pubkey.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "patient_seed" => {
                while let Some(chunk) = field.try_next().await? {
                    patient_seed.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "name" => {
                while let Some(chunk) = field.try_next().await? {
                    name.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "blood_type" => {
                while let Some(chunk) = field.try_next().await? {
                    blood_type.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "previous_report" => {
                while let Some(chunk) = field.try_next().await? {
                    previous_report.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "ph_no" => {
                while let Some(chunk) = field.try_next().await? {
                    ph_no.push_str(&String::from_utf8_lossy(chunk.as_ref()));
                }
            }
            "file" => {
                while let Some(chunk) = field.try_next().await? {
                    file_data.extend_from_slice(chunk.as_ref());
                }
            }
            _ => {}
        }
    }

    // Validate inputs
    if user_pubkey.is_empty() {
        user_pubkey = user_pubkey_from_token;
    } else if user_pubkey != user_pubkey_from_token {
        error!("User pubkey from form ({}) does not match JWT ({})", user_pubkey, user_pubkey_from_token);
        return Err(AppError::Unauthorized("User pubkey mismatch".to_string()));
    }
    if patient_seed.is_empty() {
        error!("No patient_seed provided in update_patient request");
        return Err(AppError::BadRequest("No patient_seed provided".to_string()));
    }
    if name.is_empty() || blood_type.is_empty() || previous_report.is_empty() || ph_no.is_empty() {
        error!("Missing required patient data fields");
        return Err(AppError::BadRequest("Missing required patient data fields".to_string()));
    }

    let patient_data = crate::models::PatientData {
        name,
        blood_type,
        previous_report,
        ph_no,
        file: None, // Will be updated with CID if file is uploaded
    };

    let modified_req = crate::models::UpdatePatientRequest {
        user_pubkey,
        patient_seed,
        patient_data,
    };

    let prepared_tx = if file_data.is_empty() {
        data.solana_service.prepare_update_patient(&modified_req, None).await?
    } else {
        data.solana_service.prepare_update_patient(&modified_req, Some(&file_data)).await?
    };

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
    Ok(HttpResponse::Ok().json(decrypted_data))
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