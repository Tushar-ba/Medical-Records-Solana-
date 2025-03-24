use actix_web::{
    dev::{ServiceRequest, ServiceResponse, Transform},
    web,
    Error, HttpMessage,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let app_state = match req.app_data::<web::Data<AppState>>() {
        Some(state) => state,
        None => {
            return Err((
                AppError::InternalServerError("App state not found".to_string()).into(),
                req,
            ));
        }
    };

    let config = &app_state.jwt_config;
    let token = credentials.token();

    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(e) => {
            return Err((
                AppError::Unauthorized(format!("Invalid token: {}", e)).into(),
                req,
            ));
        }
    };

    let req = req;
    req.extensions_mut()
        .insert(token_data.claims.sub.clone());
    Ok(req)
}

pub fn generate_jwt(public_key: &str, secret: &str, expires_in: i64) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(expires_in))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: public_key.to_string(),
        exp: expiration,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to generate JWT: {}", e)))?;

    Ok(token)
}

pub fn jwt_middleware<S>() -> impl Transform<
    S,
    ServiceRequest,
    Response = ServiceResponse<actix_web::body::EitherBody<actix_web::body::BoxBody>>,
    Error = Error,
    InitError = (),
>
where
    S: actix_web::dev::Service<
        ServiceRequest,
        Response = ServiceResponse<actix_web::body::BoxBody>,
        Error = Error,
    > + 'static,
    S::Future: 'static,
{
    actix_web_httpauth::middleware::HttpAuthentication::bearer(|req, credentials| {
        Box::pin(validator(req, credentials))
    })
}