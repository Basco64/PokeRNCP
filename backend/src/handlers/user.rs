use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::auth::{hash_password, verify_password};
use crate::helpers::{ApiResult, created, not_found, ok, to_500, unique_or_500};
use crate::models::user::{CreateUser, LoginUser, UpdateUser, User};

// COMPTE

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> ApiResult<(StatusCode, String)> {
    let hashed = hash_password(&payload.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hash failed".into()))?;

    sqlx::query_scalar!(
        r#"
        INSERT INTO users (username, email, password)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        payload.username,
        payload.email,
        hashed
    )
    .fetch_one(&pool)
    .await
    .map_err(unique_or_500)?;

    created("User created.")
}

pub async fn get_user(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Json<User>> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, password, created_at FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| not_found("User not found."))?;

    Ok(Json(user))
}

pub async fn update_user(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUser>,
) -> ApiResult<(StatusCode, String)> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    if let Some(ref password) = payload.password {
        let hashed = hash_password(password)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hash failed.".into()))?;
        sqlx::query!(
            "UPDATE users SET password = $1 WHERE id = $2",
            hashed,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(to_500)?;
    }
    if let Some(ref username) = payload.username {
        sqlx::query!(
            "UPDATE users SET username = $1 WHERE id = $2",
            username,
            user_id
        )
        .execute(&pool)
        .await
        .map_err(to_500)?;
    }
    if let Some(ref email) = payload.email {
        sqlx::query!("UPDATE users SET email = $1 WHERE id = $2", email, user_id)
            .execute(&pool)
            .await
            .map_err(to_500)?;
    }
    ok("User updated.")
}

pub async fn delete_user(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<(StatusCode, String)> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    let res = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .map_err(to_500)?;
    if res.rows_affected() == 0 {
        return Err(not_found("User not found"));
    }
    ok("User deleted.")
}

// CONNECTION

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
    let row = match sqlx::query!(
        r#"SELECT id, password FROM users WHERE username = $1 OR email = $1"#,
        payload.username
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let Some(row) = row else {
        return crate::helpers::unauthorized("Identifiants invalides").into_response();
    };

    if !verify_password(&row.password, &payload.password) {
        return crate::helpers::unauthorized("Identifiants invalides").into_response();
    }

    // Génère les tokens
    let access = match crate::auth::generate_access_token(row.id) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let refresh = match crate::auth::generate_refresh_token(row.id) {
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
        .map_or(false, |v| v == "true");
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
    let Some(token) = get_cookie(&headers, "refresh") else {
        return crate::helpers::unauthorized("Absence de refresh token").into_response();
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
        .map_or(false, |v| v == "true");

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
        .map_or(false, |v| v == "true");
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
