use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use backend::{app_state::AppState, config, routes};
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize env_logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    dotenv::dotenv().ok();
    let config = config::Config::from_env();

    let app_data = web::Data::new(AppState {
        config: config.clone().into(),
    });

    println!("Server running on http://{}:{}", config.server_host, config.port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        // Configure the logger middleware with a custom format
        let logger = Logger::default();

        App::new()
            .app_data(app_data.clone())
            .wrap(cors)
            .wrap(logger) // Add the logger middleware
            .configure(routes::init_routes)
    })
    .bind((config.server_host, config.port))?
    .run()
    .await
}