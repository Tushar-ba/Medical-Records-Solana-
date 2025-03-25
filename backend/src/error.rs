use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::Error as SerdeError;
use solana_client::client_error::ClientError;
use solana_sdk::pubkey::ParsePubkeyError;
use std::array::TryFromSliceError; // Corrected import
use std::io::Error as IoError;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    InternalServerError(String),
    SolanaError(String),
    InvalidProgramId(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            AppError::SolanaError(msg) => write!(f, "Solana Error: {}", msg),
            AppError::InvalidProgramId(msg) => write!(f, "Invalid Program ID: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SolanaError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidProgramId(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<ClientError> for AppError {
    fn from(error: ClientError) -> Self {
        AppError::SolanaError(error.to_string())
    }
}

impl From<SerdeError> for AppError {
    fn from(error: SerdeError) -> Self {
        AppError::InternalServerError(format!("Serialization error: {}", error))
    }
}

impl From<bs58::decode::Error> for AppError {
    fn from(error: bs58::decode::Error) -> Self {
        AppError::BadRequest(format!("Base58 decode error: {}", error))
    }
}

impl From<TryFromSliceError> for AppError {
    fn from(error: TryFromSliceError) -> Self {
        AppError::BadRequest(format!("Signature conversion error: {}", error))
    }
}

impl From<ParsePubkeyError> for AppError {
    fn from(error: ParsePubkeyError) -> Self {
        AppError::BadRequest(format!("Public key parse error: {}", error))
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(error: base64::DecodeError) -> Self {
        AppError::BadRequest(format!("Base64 decode error: {}", error))
    }
}

impl From<Box<bincode::ErrorKind>> for AppError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        AppError::InternalServerError(format!("Bincode deserialization error: {}", error))
    }
}

impl From<IoError> for AppError {
    fn from(error: IoError) -> Self {
        AppError::InternalServerError(format!("IO error during deserialization: {}", error))
    }
}