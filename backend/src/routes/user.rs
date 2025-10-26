use crate::handlers::user::{create_user, delete_user, get_user, get_users, update_user};
use axum::Router;
use axum::routing::{delete, get, patch, post};
use sqlx::PgPool;

pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .route("/get_user/{id}", get(get_user))
        .route("/update_user/{id}", patch(update_user))
        .route("/delete_user/{id}", delete(delete_user))
}
