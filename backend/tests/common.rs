use axum::{Router, routing::get};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;

static POOL: OnceCell<PgPool> = OnceCell::const_new();

pub async fn test_pool() -> &'static PgPool {
    let _ = dotenvy::dotenv();

    POOL.get_or_init(|| async {
        let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL for tests");

        eprintln!("[tests] Using TEST_DATABASE_URL={}", url);

        let pool = PgPool::connect(&url).await.expect("DB connect failed");
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
fn health_app() -> Router {
    Router::new().route("/", get(|| async { "Bienvenue sur le pokeRncp" }))
}

#[allow(dead_code)]
pub async fn start_server() -> (String, tokio::task::JoinHandle<()>) {
    let app = health_app();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("serve error: {e}");
        }
    });

    (url, handle)
}
