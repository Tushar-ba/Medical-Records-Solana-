use std::sync::Mutex;
use crate::config::Config; // Updated import
use crate::services::SolanaService; // Updated import

pub struct AppState {
    pub config: Mutex<Config>,
    pub solana_service: SolanaService,
}