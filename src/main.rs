use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::{http::header, web, App, HttpServer};
use config::Configs;
use diesel_migrations::EmbeddedMigrations;
use dotenvy::dotenv;
use env_logger::Env;
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

mod auth_guard;
mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod routes;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::new().default_filter_or("info"));

    log::info!("Starting the server...");

    let pool = db::init_pool();
    log::info!("Database pool initialized.");

    log::info!("checking if migration are upto date?");
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    let _ = utils::run_migrations(&mut conn);
    log::info!("All migration are upto date!");

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(50)
        .burst_size(60)
        .finish()
        .unwrap();
    log::info!("Governor configuration created.");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Governor::new(&governor_conf))
            .wrap(Logger::default()) // Middleware for logging
            .wrap(
                DefaultHeaders::new()
                    .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                    .add((header::X_FRAME_OPTIONS, "DENY"))
                    .add((header::X_XSS_PROTECTION, "1; mode=block"))
                    .add((header::CONTENT_SECURITY_POLICY, "default-src 'self'"))
                    .add((header::REFERRER_POLICY, "no-referrer")),
            )
            .configure(routes::init)
    });

    let port = Configs::get_app_configs().port;
    log::info!("Server will start on port: {}", port);

    server.bind(format!("0.0.0.0:{}", port))?.run().await
}
