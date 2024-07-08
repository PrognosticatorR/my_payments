use actix_web::web;

use crate::handlers::user::*;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/{id}", web::get().to(get_profile))
            .route("/{id}", web::put().to(update_profile))
            .route("/{id}/reset_password", web::patch().to(update_password))
            .route("/{id}/update_balance", web::put().to(update_balance))
            .route("/{id}", web::delete().to(delete_user)),
    );
}
