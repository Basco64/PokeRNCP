use crate::handlers::user::{create_user, delete_user, update_user};
use axum::Router;
use axum::routing::{delete, patch, post};
use sqlx::PgPool;

pub fn user_routes() -> Router<PgPool> {
    Router::new()
        .route("/", post(create_user))
        .route("/{id}", patch(update_user))
        .route("/{id}", delete(delete_user))
}
