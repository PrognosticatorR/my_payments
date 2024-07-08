#![allow(dead_code)]
use std::env;
pub struct Configs {
    pub database_url: String,
    pub port: String,
    pub jwt_secret: String,
    pub app_host: String,
}

impl Configs {
    pub fn get_app_configs() -> Self {
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:mysecretpassword@localhost:5432/postgres".to_string()
        });
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "my_secret".to_string());
        let app_host =
            env::var("APP_HOST").unwrap_or_else(|_| "http://localhost:8080/api/v1.0".to_string());

        Self {
            database_url,
            port,
            jwt_secret,
            app_host,
        }
    }
}
