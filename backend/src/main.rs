use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod app_state;
mod controllers;
mod error;
mod middleware; // Ensure this is present
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

    let app_state = web::Data::new(app_state::AppState {
        solana_service: services::TransactionService::new(&rpc_url, admin_keypair, &program_id)
            .expect("Failed to create TransactionService"),
        jwt_config: models::JwtConfig {
            secret: jwt_secret,
            expires_in: jwt_expires_in,
        },
    });

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
                            .route("/submit", web::post().to(controllers::submit_transaction)),
                    ),
            )
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}