use actix_web::dev::Payload;
use actix_web::http::header::HeaderValue;
use actix_web::{Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};

use crate::utils::{verify_jwt, Claims};

#[derive(Debug)]
pub struct AuthGuard {
    pub claims: Claims,
}

impl Claims {
    pub fn has_role(&self, required_role: &str) -> bool {
        self.role == required_role
    }
}
impl FromRequest for AuthGuard {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        const BEARER_PREFIX: &str = "Bearer ";

        let auth_header: Option<&HeaderValue> = req.headers().get("Authorization");

        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix(BEARER_PREFIX) {
                    return match verify_jwt(token) {
                        Ok(token_data) => ready(Ok(AuthGuard { claims: token_data })),
                        Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
                    };
                }
            }
        }

        ready(Err(actix_web::error::ErrorUnauthorized(
            "No token provided",
        )))
    }
}
