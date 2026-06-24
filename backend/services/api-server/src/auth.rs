use actix_web::{dev, http::header, FromRequest, HttpRequest, ResponseError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::{ready, Ready};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub username: String,
    pub exp: i64, // expiration timestamp
    pub iat: i64, // issued at timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqUser {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug)]
pub enum AuthError {
    MissingHeader,
    InvalidHeader,
    TokenExpired,
    InvalidToken,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::MissingHeader => write!(f, "Missing Authorization header"),
            AuthError::InvalidHeader => write!(f, "Invalid Authorization header format"),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::InvalidToken => write!(f, "Invalid token signature or claims"),
        }
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            AuthError::MissingHeader | AuthError::InvalidHeader => {
                actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            AuthError::TokenExpired => {
                actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "token_expired",
                    "message": self.to_string()
                }))
            }
            AuthError::InvalidToken => {
                actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "invalid_token",
                    "message": self.to_string()
                }))
            }
        }
    }
}

/// Extract ReqUser from Request Headers using JWT Bearer Token
impl FromRequest for ReqUser {
    type Error = AuthError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let auth_header = req.headers().get(header::AUTHORIZATION);
        if auth_header.is_none() {
            return ready(Err(AuthError::MissingHeader));
        }

        let auth_str = match auth_header.unwrap().to_str() {
            Ok(s) => s,
            Err(_) => return ready(Err(AuthError::InvalidHeader)),
        };

        if !auth_str.starts_with("Bearer ") {
            return ready(Err(AuthError::InvalidHeader));
        }

        // Extract the token from the header starting from 7th index
        let token = &auth_str[7..];

        // Retrieve JWT Secret from app config/state or environment
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            "super_secret_unbreakable_ledger_key_change_me_in_production".to_string()
        });

        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let mut validation = Validation::default();
        // Since we run on regtest/local clock might deviate, disable time validation or configure clock skew:
        validation.leeway = 60; // 60 seconds leeway

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                if claims.exp < Utc::now().timestamp() {
                    ready(Err(AuthError::TokenExpired))
                } else {
                    ready(Ok(ReqUser {
                        id: claims.sub,
                        username: claims.username,
                    }))
                }
            }
            Err(_) => ready(Err(AuthError::InvalidToken)),
        }
    }
}

/// Hash a password using Argon2id algorithm
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

/// Verify a password against an Argon2id hash
pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Generate JWT access token for a user
pub fn generate_jwt(
    user_id: Uuid,
    username: &str,
    secret: &str,
    expires_in_secs: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + Duration::seconds(expires_in_secs);
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: expiration.timestamp(),
        iat: now.timestamp(),
    };

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &encoding_key)
}
