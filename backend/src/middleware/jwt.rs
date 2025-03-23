use actix_web::{dev::ServiceRequest, HttpMessage};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use log;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Wallet public key
    pub exp: i64,
}

pub async fn jwt_middleware(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let app_state = req
        .app_data::<actix_web::web::Data<crate::app_state::AppState>>()
        .expect("AppState not found");

    let claims = {
        let config = app_state.config.lock().unwrap();
        let jwt_secret = config.jwt_secret.as_bytes();

        let token = credentials.token();
        let validation = Validation::default();

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret),
            &validation,
        )
    };

    match claims {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(e) => {
            log::error!("JWT validation failed: {:?}", e);
            let config = Config::default();
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}