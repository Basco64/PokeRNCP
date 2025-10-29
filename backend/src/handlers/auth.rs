use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;
use sqlx::PgPool;

use crate::auth::{CurrentUser, hash_password, verify_password};
use crate::models::auth::{
    ChangePasswordPayload, ConfirmPasswordResetPayload, RequestPasswordResetPayload,
};
use crate::models::user::LoginUser;

fn get_bearer(headers: &HeaderMap) -> Option<String> {
    let v = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    v.strip_prefix("Bearer ").map(|s| s.to_string())
}

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

pub async fn login_user(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginUser>,
) -> impl IntoResponse {
    let row = match sqlx::query_as::<_, (uuid::Uuid, String)>(
        r#"SELECT id, password FROM users WHERE username = $1 OR email = $1"#,
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let Some((user_id, password_hash)) = row else {
        return crate::helpers::unauthorized("Identifiants invalides").into_response();
    };

    if !verify_password(&password_hash, &payload.password) {
        return crate::helpers::unauthorized("Identifiants invalides").into_response();
    }

    // Génère les tokens
    let access = match crate::auth::generate_access_token(user_id) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let refresh = match crate::auth::generate_refresh_token(user_id) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let access_max = std::env::var("JWT_EXP_SECONDS")
        .ok()
        .unwrap_or_else(|| "900".into());
    let refresh_max = std::env::var("JWT_REFRESH_EXP_SECONDS")
        .ok()
        .unwrap_or_else(|| "2592000".into());

    let prod = std::env::var("PRODUCTION_MODE")
        .ok()
        .is_some_and(|v| v == "true");
    let secure_flag = if prod { "; Secure" } else { "" };

    let access_cookie =
        format!("auth={access}; Path=/; Max-Age={access_max}; HttpOnly; SameSite=Lax{secure_flag}");
    let refresh_cookie = format!(
        "refresh={refresh}; Path=/; Max-Age={refresh_max}; HttpOnly; SameSite=Strict{secure_flag}"
    );

    let mut res = "Connexion réussie.".to_string().into_response();
    res.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&access_cookie).unwrap(),
    );
    res.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&refresh_cookie).unwrap(),
    );
    *res.status_mut() = StatusCode::OK;
    res
}

pub async fn refresh_token(headers: HeaderMap) -> impl IntoResponse {
    let token = get_bearer(&headers).or_else(|| get_cookie(&headers, "refresh"));
    let Some(token) = token else {
        return crate::helpers::unauthorized("Refresh token requis").into_response();
    };

    let claims = match crate::auth::verify_refresh(&token) {
        Ok(c) => c,
        Err(_) => return crate::helpers::unauthorized("Refresh token invalide").into_response(),
    };

    let access = match crate::auth::generate_access_token(claims.sub) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let access_max = std::env::var("JWT_EXP_SECONDS")
        .ok()
        .unwrap_or_else(|| "900".into());
    let prod = std::env::var("PRODUCTION_MODE")
        .ok()
        .is_some_and(|v| v == "true");
    let secure_flag = if prod { "; Secure" } else { "" };

    let access_cookie =
        format!("auth={access}; Path=/; Max-Age={access_max}; HttpOnly; SameSite=Lax{secure_flag}");

    let mut res = "Token régénéré.".to_string().into_response();
    res.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&access_cookie).unwrap(),
    );
    *res.status_mut() = StatusCode::OK;
    res
}
pub async fn logout_user() -> impl IntoResponse {
    let prod = std::env::var("PRODUCTION_MODE")
        .ok()
        .is_some_and(|v| v == "true");
    let secure_flag = if prod { "; Secure" } else { "" };

    let mut res = "Déconnecté.".to_string().into_response();
    res.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "auth=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{secure_flag}"
        ))
        .unwrap(),
    );
    res.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "refresh=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict{secure_flag}"
        ))
        .unwrap(),
    );
    *res.status_mut() = StatusCode::OK;
    res
}

pub async fn me(
    State(pool): State<PgPool>,
    CurrentUser(user_id): CurrentUser,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, (uuid::Uuid, String, Option<String>)>(
        r#"SELECT id, username, email FROM users WHERE id = $1"#,
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await;
    match row {
        Ok(Some((id, username, email))) => (
            StatusCode::OK,
            Json(json!({
                "id": id,
                "username": username,
                "email": email
            })),
        )
            .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Utilisateur introuvable").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn change_password(
    State(pool): State<PgPool>,
    CurrentUser(user_id): CurrentUser,
    Json(payload): Json<ChangePasswordPayload>,
) -> impl IntoResponse {
    let row = match sqlx::query_scalar::<_, String>(r#"SELECT password FROM users WHERE id = $1"#)
        .bind(user_id)
        .fetch_optional(&pool)
        .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let Some(current_hash) = row else {
        return (StatusCode::NOT_FOUND, "Utilisateur introuvable").into_response();
    };

    if !verify_password(&current_hash, &payload.current_password) {
        return crate::helpers::unauthorized("Mot de passe actuel incorrect").into_response();
    }

    let new_hash = match hash_password(&payload.new_password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if let Err(e) = sqlx::query(r#"UPDATE users SET password = $1 WHERE id = $2"#)
        .bind(new_hash)
        .bind(user_id)
        .execute(&pool)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    (StatusCode::OK, "Mot de passe mis à jour").into_response()
}

pub async fn request_password_reset(
    State(pool): State<PgPool>,
    Json(payload): Json<RequestPasswordResetPayload>,
) -> impl IntoResponse {
    let prod = std::env::var("PRODUCTION_MODE")
        .ok()
        .is_some_and(|v| v == "true");

    let user = (sqlx::query_scalar::<_, uuid::Uuid>(
        r#"SELECT id FROM users WHERE email = $1 OR username = $1"#,
    )
    .bind(&payload.email_or_username)
    .fetch_optional(&pool)
    .await)
        .unwrap_or_default();

    if let Some(u) = user
        && let Ok(token) = crate::auth::generate_reset_token(u)
    {
        // En prod: envoi par mail plutot.
        if !prod {
            return (StatusCode::ACCEPTED, Json(json!({ "reset_token": token }))).into_response();
        }
        // TODO: mettre ne place envoi du mail
    }

    (StatusCode::ACCEPTED, Json(json!({ "status": "ok" }))).into_response()
}

pub async fn confirm_password_reset(
    State(pool): State<PgPool>,
    Json(payload): Json<ConfirmPasswordResetPayload>,
) -> impl IntoResponse {
    let claims = match crate::auth::verify_reset(&payload.token) {
        Ok(c) => c,
        Err(_) => return (StatusCode::BAD_REQUEST, "Token invalide ou expiré").into_response(),
    };

    if payload.new_password.len() < 8 {
        return (StatusCode::BAD_REQUEST, "Mot de passe trop court").into_response();
    }

    let new_hash = match hash_password(&payload.new_password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if let Err(e) = sqlx::query(r#"UPDATE users SET password = $1 WHERE id = $2"#)
        .bind(new_hash)
        .bind(claims.sub)
        .execute(&pool)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    (StatusCode::OK, "Mot de passe réinitialisé").into_response()
}
