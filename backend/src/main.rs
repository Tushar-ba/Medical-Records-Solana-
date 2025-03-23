use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use backend::{app_state::AppState, config::Config, routes, services::SolanaService}; // Updated imports
use solana_client::rpc_client::RpcClient;
use std::time::Duration;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    dotenv::dotenv().ok();
    let config = Config::from_env(); // Updated reference

    let solana_client = RpcClient::new_with_timeout(config.solana_url.clone(), Duration::from_secs(30));

    let solana_service = SolanaService::new( // Updated reference
        solana_client,
        "Bgsncv4N8H6oWjYHa9KaxCKaWcwKxCMa9FHDHsGkAUzW",
        &config.admin_pubkey,
    ).expect("Failed to initialize SolanaService");

    let app_data = web::Data::new(AppState {
        config: std::sync::Mutex::new(config.clone()),
        solana_service,
    });

    println!("Server running on http://{}:{}", config.server_host, config.port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        let logger = Logger::default();

        App::new()
            .app_data(app_data.clone())
            .wrap(cors)
            .wrap(logger)
            .configure(routes::init_routes)
    })
    .bind((config.server_host, config.port))?
    .run()
    .await
}