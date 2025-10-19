use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers::pokemon::{
    create_pokemon, delete_pokemon, get_pokemon, list_pokemons, update_pokemon,
};

pub fn pokemon_routes() -> Router<PgPool> {
    Router::new()
        .route(
            "/users/{user_id}/pokemons",
            get(list_pokemons).post(create_pokemon),
        )
        .route(
            "/users/{user_id}/pokemons/{capture_id}",
            get(get_pokemon)
                .patch(update_pokemon)
                .delete(delete_pokemon),
        )
}
