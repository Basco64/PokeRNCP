use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct RequestPasswordResetPayload {
    pub email_or_username: String,
}

#[derive(Deserialize)]
pub struct ConfirmPasswordResetPayload {
    pub token: String,
    pub new_password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResetClaims {
    pub sub: Uuid,
    pub iat: i64,
    pub exp: i64,
    pub scope: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iat: i64,
    pub exp: i64,
}
