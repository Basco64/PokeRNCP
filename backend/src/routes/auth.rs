use crate::handlers::auth::{
    change_password, confirm_password_reset, login_user, logout_user, me, refresh_token,
    request_password_reset,
};
use axum::Router;
use axum::routing::{get, post, put};
use sqlx::PgPool;

pub fn user_routes() -> Router<PgPool> {
    Router::new()
        //PUBLIC
        .route("/login", post(login_user))
        .route("/refresh-token", post(refresh_token))
        .route("/logout", post(logout_user))
        //PRIVE
        .route("/request-password-reset", post(request_password_reset))
        .route("/confirm-password-reset", post(confirm_password_reset))
        .route("/me", get(me))
        .route("/change-password", put(change_password))
}
