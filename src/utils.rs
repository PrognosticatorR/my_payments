use std::error::Error;

use actix_web::HttpResponse;
use chrono::{Duration, Utc};

use diesel::pg::Pg;
use diesel_migrations::MigrationHarness;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
extern crate diesel;

use crate::{config::Configs, models::user::User, MIGRATIONS};
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub role: String,
    pub id: Uuid,
    pub exp: usize,
}

pub fn generate_jwt(user: User) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(3)) // 3 days
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        id: user.id,
        email: user.email,
        exp: expiration as usize,
        role: user.role,
    };
    let jwt_secret = Configs::get_app_configs().jwt_secret;

    let key = EncodingKey::from_secret(jwt_secret.as_ref());
    encode(&Header::default(), &claims, &key)
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let jwt_secret = Configs::get_app_configs().jwt_secret;

    let key = DecodingKey::from_secret(jwt_secret.as_ref());
    decode::<Claims>(token, &key, &validation).map(|data| data.claims)
}

pub fn create_response<T: Serialize>(
    success: bool,
    status: u16,
    message: Option<&str>,
    data: Option<T>,
) -> HttpResponse {
    let mut response_body = json!({
        "success": success,
    });
    if let Some(msg) = message {
        response_body["message"] = json!(msg);
    }
    if let Some(d) = data {
        response_body["data"] = json!(d);
    }
    HttpResponse::build(actix_web::http::StatusCode::from_u16(status).unwrap()).json(response_body)
}

pub fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
