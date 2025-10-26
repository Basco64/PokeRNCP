use crate::routes;
use axum::{Router, routing::get};
use sqlx::PgPool;

pub fn build_routes() -> Router<PgPool> {
    Router::new()
        .route("/api", get(|| async { "Bienvenue sur le pokeRncp" }))
        .nest("/api/auth", routes::auth::user_routes())
        .nest("/api/users", routes::user::user_routes())
        .nest("/api/pokemons", routes::pokemon::pokemon_routes())
}
