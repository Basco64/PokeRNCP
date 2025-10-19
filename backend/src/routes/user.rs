use crate::handlers::user::{
    create_user, delete_user, get_user, login_user, logout_user, refresh_token, update_user,
};
use axum::Router;
use axum::routing::{delete, get, patch, post};
use sqlx::PgPool;

pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/create_user", post(create_user))
        .route("/get_user/{id}", get(get_user))
        .route("/update_user/{id}", patch(update_user))
        .route("/delete_user/{id}", delete(delete_user))
        .route("/login", post(login_user))
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout_user))
}
