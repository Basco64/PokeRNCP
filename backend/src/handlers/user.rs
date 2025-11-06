use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{CurrentUser, hash_password};
use crate::helpers::{
    ApiResult, conflict, created, internal_server_error, not_found, ok, to_500, unauthorized,
};
use crate::models::user::{CreateUser, UpdateUser};

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> ApiResult<(StatusCode, String)> {
    let hashed =
        hash_password(&payload.password).map_err(|_| internal_server_error("Hash failed"))?;

    let res: Result<Uuid, sqlx::Error> = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, email, password)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&hashed)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(_) => {}
        Err(e) => {
            if let sqlx::Error::Database(db) = &e
                && db.code().as_deref() == Some("23505")
            {
                return Err(conflict("⚠️ User already exists."));
            }
            return Err(to_500(e));
        }
    }

    created("User created.")
}

pub async fn update_user(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUser>,
) -> ApiResult<(StatusCode, String)> {
    if current_user != user_id {
        return Err(unauthorized("ACCESS DENIED"));
    }
    if let Some(ref password) = payload.password {
        let hashed = hash_password(password).map_err(|_| internal_server_error("Hash failed."))?;
        sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
            .bind(&hashed)
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(to_500)?;
    }
    if let Some(ref username) = payload.username {
        sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
            .bind(username)
            .bind(user_id)
            .execute(&pool)
            .await
            .map_err(to_500)?;
    }
    if let Some(ref email) = payload.email {
        sqlx::query("UPDATE users SET email = $1 WHERE id = $2")
            .bind(email)
            .bind(user_id)
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
        return Err(unauthorized("ACCESS DENIED"));
    }
    let res = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(to_500)?;
    if res.rows_affected() == 0 {
        return Err(not_found("User not found"));
    }
    ok("User deleted.")
}
