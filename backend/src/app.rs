// use axum::http::{HeaderValue, Method};
// use axum::{Router, routing::get};
// use sqlx::PgPool;
// use tower_http::cors::CorsLayer;

// use crate::routes;

// pub fn build_app(db_pool: PgPool) -> Router<PgPool> {
//     let origin =
//         std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());

//     let cors = CorsLayer::new()
//         .allow_origin(origin.parse::<HeaderValue>().unwrap())
//         .allow_methods([
//             Method::GET,
//             Method::POST,
//             Method::PUT,
//             Method::PATCH,
//             Method::DELETE,
//             Method::OPTIONS,
//         ])
//         .allow_headers([
//             axum::http::header::CONTENT_TYPE,
//             axum::http::header::AUTHORIZATION,
//         ])
//         .allow_credentials(true);

//     Router::new()
//         .route("/", get(|| async { "Bienvenue sur le pokeRncp" }))
//         .merge(routes::user::user_routes())
//         .merge(routes::pokemon::pokemon_routes())
//         .with_state(db_pool)
//         .layer(cors)
// }

use crate::routes;
use axum::{Router, routing::get};
use sqlx::PgPool;

pub fn build_routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(|| async { "Bienvenue sur le pokeRncp" }))
        .merge(routes::user::user_routes())
        .merge(routes::pokemon::pokemon_routes())
}
