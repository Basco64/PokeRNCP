use axum::Router;
use axum::routing::get;

use crate::db::init_db;
use dotenvy::dotenv;
use tokio::net::TcpListener;

mod auth;
mod db;
mod handlers;
mod helpers;
mod models;
mod routes;

// cargo watch -c -x run

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_pool = init_db().await;
    let addr = std::env::var("BACKEND_URL").expect("BACKEND_URL must be set.");

    let app = Router::new()
        .route("/", get(|| async { "Bienvenue sur le pokeRncp" }))
        .merge(routes::user::user_routes())
        .with_state(db_pool); // - with_state(db_pool) rend le pool accessible aux handlers via l'état

    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("🚀 Serveur démarré sur http://{addr}");

    axum::serve(listener, app).into_future().await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use sqlx;
    use tower::util::ServiceExt;

    fn build_test_app() -> Router {
        // Router indépendant de la DB
        Router::new().route("/", get(|| async { "Bienvenue sur le pokeRncp" }))
    }

    #[tokio::test]
    async fn get_racine_retourne_bienvenue() {
        let app = build_test_app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "text/plain; charset=utf-8");

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "Bienvenue sur le pokeRncp"
        );
    }

    #[tokio::test]
    async fn post_racine_retourne_405() {
        let app = build_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

        let allow = response
            .headers()
            .get(header::ALLOW)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(allow.contains("GET"));
    }

    #[tokio::test]
    async fn route_inconnue_retourne_404() {
        let app = build_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn base_de_donnees_repond() {
        dotenv().ok();
        // let database_url = std::env::var("TEST_DATABASE_URL")
        //     .or_else(|_| std::env::var("DATABASE_URL"))
        //     .expect("DATABASE_URL or TEST_DATABASE_URL must be set.");

        let pool = init_db().await;
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let one: i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("La base ne répond pas");

        assert_eq!(one, 1);
    }
}
