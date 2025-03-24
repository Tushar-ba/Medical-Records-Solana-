use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::{controllers, middleware::jwt};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    let jwt_middleware = HttpAuthentication::bearer(jwt::validator);

    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/auth")
                    .route(web::post().to(controllers::authenticate)),
            )
            .service(
                web::resource("/transactions/prepare/add-read-authority")
                    .wrap(jwt_middleware.clone())
                    .route(web::post().to(controllers::prepare_add_read_authority)),
            )
            .service(
                web::resource("/transactions/submit")
                    .wrap(jwt_middleware.clone())
                    .route(web::post().to(controllers::submit_transaction)),
            )
            .service(
                web::resource("/transactions/authorities")
                    .wrap(jwt_middleware.clone())
                    .route(web::get().to(controllers::get_authorities)),
            )
            .service(
                web::resource("/protected")
                    .wrap(jwt_middleware)
                    .route(web::get().to(protected_route)),
            ),
    );
}

async fn protected_route() -> impl Responder {
    HttpResponse::Ok().body("This is a protected route under /api/protected!")
}