use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use std::fs;
use serde_json;

mod app_state;
mod controllers;
mod error;
mod middleware;
mod models;
mod services;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let rpc_url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set");
    let admin_keypair = env::var("ADMIN_KEYPAIR").expect("ADMIN_KEYPAIR must be set");
    let program_id = env::var("PROGRAM_ID").expect("PROGRAM_ID must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_expires_in: i64 = env::var("JWT_EXPIRES_IN")
        .unwrap_or("3600".to_string())
        .parse()
        .expect("JWT_EXPIRES_IN must be a number");

    let admin_keypair: solana_sdk::signature::Keypair = {
        let bytes: Vec<u8> = serde_json::from_str(&admin_keypair).expect("Invalid ADMIN_KEYPAIR format");
        solana_sdk::signature::Keypair::from_bytes(&bytes).expect("Failed to create Keypair from bytes")
    };

    let solana_service = services::TransactionService::new(&rpc_url, admin_keypair, &program_id)
        .expect("Failed to create TransactionService");
    let jwt_config = models::JwtConfig {
        secret: jwt_secret,
        expires_in: jwt_expires_in,
    };
    let app_state = web::Data::new(app_state::AppState::new(solana_service, jwt_config));

    // Load patient_seed_map from file
    let seed_map_file = "patient_seed_map.json";
    if let Ok(file_contents) = fs::read_to_string(seed_map_file) {
        if let Ok(seed_map_data) = serde_json::from_str::<Vec<(String, String)>>(&file_contents) {
            for (pda, seed) in seed_map_data {
                app_state.patient_seed_map.insert(pda, seed);
            }
            log::info!("Loaded patient_seed_map with {} entries from {}", app_state.patient_seed_map.len(), seed_map_file);
        } else {
            log::warn!("Failed to parse patient_seed_map from {}, starting with empty map", seed_map_file);
        }
    } else {
        log::info!("No patient_seed_map file found at {}, starting with empty map", seed_map_file);
    }

    log::info!("Server running on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .route("/auth", web::post().to(controllers::authenticate))
                    .service(
                        web::scope("/transactions")
                            .wrap(middleware::jwt::jwt_middleware())
                            .route("/prepare/add-read-authority", web::post().to(controllers::prepare_add_read_authority))
                            .route("/prepare/remove-read-authority", web::post().to(controllers::prepare_remove_read_authority))
                            .route("/prepare/add-write-authority", web::post().to(controllers::prepare_add_write_authority))
                            .route("/prepare/remove-write-authority", web::post().to(controllers::prepare_remove_write_authority))
                            .route("/prepare/create-patient", web::post().to(controllers::prepare_create_patient))
                            .route("/prepare/update-patient", web::post().to(controllers::prepare_update_patient))
                            .route("/submit", web::post().to(controllers::submit_transaction))
                            .route("/authorities", web::get().to(controllers::get_authorities))
                            .route("/authority-history", web::get().to(controllers::get_authority_history))
                    )
                    .service(
                        web::scope("")
                            .wrap(middleware::jwt::jwt_middleware())
                            .route("/patient/{patient_seed}", web::get().to(controllers::get_patient))
                            .route("/view_patient/{token}", web::get().to(controllers::view_patient))
                            .route("/patients/addresses", web::get().to(controllers::get_patient_addresses))
                    ),
            )
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}