use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers::pokemon::{catch, get_pokemon_by_id, list_all, search_pokemons};

pub fn pokemon_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_all))
        .route("/search", get(search_pokemons))
        .route("/catch", post(catch))
        .route("/{pokemon_id}", get(get_pokemon_by_id))
}
