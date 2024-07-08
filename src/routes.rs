use actix_web::web;

pub mod transaction;
pub mod user;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(user::init)
            .configure(transaction::init),
    );
}
