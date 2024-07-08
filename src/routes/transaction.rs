use actix_web::web;

use crate::handlers::transaction::*;
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/transactions")
            .route("/transact", web::post().to(create_transaction))
            .route("/{id}", web::get().to(get_transaction))
            .route("/user/{user_id}", web::get().to(list_transactions)),
    );
}
