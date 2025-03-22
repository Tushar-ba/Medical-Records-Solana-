use actix_web::web;

use crate::controllers;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api").service(
            web::resource("/auth")
                .route(web::post().to(controllers::authenticate)),
        ),
    );
}