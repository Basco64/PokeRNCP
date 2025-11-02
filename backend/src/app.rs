use crate::{helpers, routes};
use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::services::{ServeDir, ServeFile};

pub fn build_routes() -> Router<PgPool> {
    let api = Router::new()
        .route("/api", get(|| async { "Bienvenue sur le pokeRncp" }))
        .nest("/api/auth", routes::auth::user_routes())
        .nest("/api/users", routes::user::user_routes())
        .nest("/api/pokemons", routes::pokemon::pokemon_routes());

    // Service des fichiers statiques (frontend Yew buildé)
    // Utilise un fallback_service à la racine pour supporter le routage SPA (Axum >= 0.8)
    let spa_service = ServeDir::new("static").fallback(ServeFile::new("static/index.html"));

    api
        // Nettoyage du contexte utilisateur au début de chaque requête
        .layer(axum::middleware::from_fn(helpers::clear_user_mw))
        // Toute requête non prise par /api tombera sur le service statique
        .fallback_service(spa_service)
}
