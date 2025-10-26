use axum::Router;
use axum::http::{HeaderValue, Method};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;

static POOL: OnceCell<PgPool> = OnceCell::const_new();

pub async fn test_pool() -> &'static PgPool {
    let _ = dotenvy::dotenv();

    POOL.get_or_init(|| async {
        let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL for tests");

        eprintln!("[tests] Using TEST_DATABASE_URL={}", url);
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(30)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&url)
            .await
            .expect("DB connect failed");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Migration failed");
        pool
    })
    .await
}

#[allow(dead_code)]
pub async fn test_build_app() -> axum::Router<PgPool> {
    let pool = test_pool().await.clone();
    pokedex_rncp_backend::app::build_routes().with_state(pool)
}

#[allow(dead_code)]
pub async fn start_server() -> (String, tokio::task::JoinHandle<()>) {
    let _ = dotenvy::dotenv();
    let url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("Set TEST_DATABASE_URL for tests");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&url)
        .await
        .expect("server DB connect failed");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migration failed (server)");
    let origin =
        std::env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    let cors = tower_http::cors::CorsLayer::new()
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

    let app: Router<_> = pokedex_rncp_backend::app::build_routes()
        .with_state(pool)
        .layer(cors);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        let server = axum::serve(listener, app)
            .with_graceful_shutdown(async { std::future::pending::<()>().await });
        if let Err(e) = server.into_future().await {
            eprintln!("serve error: {e}");
        }
    });

    for _ in 0..30 {
        if let Ok(resp) = reqwest::get(format!("{url}/api")).await {
            if resp.status().is_success() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    (url, handle)
}
