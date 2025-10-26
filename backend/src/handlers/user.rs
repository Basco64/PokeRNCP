use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::auth::hash_password;
use crate::helpers::{ApiResult, created, not_found, ok, to_500, unique_or_500};
use crate::models::user::{CreateUser, UpdateUser, User};

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

pub async fn get_users(
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
