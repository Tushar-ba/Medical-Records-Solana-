use crate::models::JwtConfig;
use crate::services::TransactionService;
use dashmap::DashMap;

pub struct AppState {
    pub solana_service: TransactionService,
    pub jwt_config: JwtConfig,
    pub token_store: DashMap<String, (String, u64)>, // (patient_seed, expiration_timestamp)
    pub patient_seed_map: DashMap<String, String>,   // (patient_pda, patient_seed)
}

impl AppState {
    pub fn new(solana_service: TransactionService, jwt_config: JwtConfig) -> Self {
        Self {
            solana_service,
            jwt_config,
            token_store: DashMap::new(),
            patient_seed_map: DashMap::new(),
        }
    }
}