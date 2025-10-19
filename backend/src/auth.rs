use argon2::{
    Argon2,
    password_hash::{
        Error as PHCError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, StatusCode, header, request::Parts};

pub fn hash_password(password: &str) -> Result<String, PHCError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iat: i64,
    pub exp: i64,
}

fn access_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

fn refresh_secret() -> String {
    std::env::var("JWT_REFRESH_SECRET").unwrap_or_else(|_| access_secret())
}

fn access_ttl_secs() -> i64 {
    std::env::var("JWT_EXP_SECONDS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(900)
}

fn refresh_ttl_secs() -> i64 {
    std::env::var("JWT_REFRESH_EXP_SECONDS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(2_592_000)
}

pub fn generate_access_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let claims = Claims {
        sub: user_id,
        iat: now,
        exp: now + access_ttl_secs(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(access_secret().as_bytes()),
    )
}

pub fn generate_refresh_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let claims = Claims {
        sub: user_id,
        iat: now,
        exp: now + refresh_ttl_secs(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(refresh_secret().as_bytes()),
    )
}

pub fn verify_access(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(access_secret().as_bytes()),
        &validation,
    )?;
    Ok(data.claims)
}

pub fn verify_refresh(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(refresh_secret().as_bytes()),
        &validation,
    )?;
    Ok(data.claims)
}

#[derive(Clone, Copy, Debug)]
pub struct CurrentUser(pub Uuid);

fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let p = part.trim();
        if let Some(v) = p.strip_prefix(&format!("{name}=")) {
            return Some(v.to_string());
        }
    }
    None
}

fn get_bearer(headers: &HeaderMap) -> Option<String> {
    let v = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    v.strip_prefix("Bearer ").map(|s| s.to_string())
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let token = get_cookie(headers, "auth")
            .or_else(|| get_bearer(headers))
            .ok_or((StatusCode::UNAUTHORIZED, "Token manquant".into()))?;

        let claims = verify_access(&token)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Token invalide".into()))?;

        Ok(CurrentUser(claims.sub))
    }
}
