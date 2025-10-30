use axum::Router;
use axum::http::{HeaderValue, Method};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use pokedex_rncp_backend::{app, db::init_db, helpers};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = init_db(&url).await;

    let mut addr = std::env::var("BACKEND_URL").expect("BACKEND_URL must be set.");
    if let Some(s) = addr
        .strip_prefix("http://")
        .or_else(|| addr.strip_prefix("https://"))
    {
        addr = s.to_string();
    }

    let origin =
        std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    let cors = CorsLayer::new()
        .allow_origin(origin.parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    let app: Router<_> = app::build_routes().with_state(db_pool).layer(cors);

    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("🚀 Serveur démarré sur {addr} (Ctrl+C pour arrêter)");

    let server = axum::serve(listener, app).with_graceful_shutdown(helpers::shutdown());
    if let Err(err) = server.into_future().await {
        eprintln!("Erreur serveur: {err}");
    }
}
