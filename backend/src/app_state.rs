use crate::models::JwtConfig;
use crate::services::TransactionService;

pub struct AppState {
    pub solana_service: TransactionService,
    pub jwt_config: JwtConfig,
}