use axum::Router;
use axum::http::{HeaderValue, Method};
use sqlx::{Connection, PgPool};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use uuid::Uuid;

static POOL: OnceCell<PgPool> = OnceCell::const_new();

pub async fn test_pool() -> &'static PgPool {
    let _ = dotenvy::dotenv();

    POOL.get_or_init(|| async {
        let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL for tests");
        eprintln!("[tests] Using TEST_DATABASE_URL={}", url);
        pokedex_rncp_backend::db::init_db(&url).await
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
    let pool = pokedex_rncp_backend::db::init_db(&url).await;
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
        if let Ok(resp) = reqwest::get(format!("{url}/api")).await
            && resp.status().is_success()
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    (url, handle)
}

#[allow(dead_code)]
pub async fn create_test_user(base: &str) -> (Uuid, String, String, String) {
    let _ = dotenvy::dotenv();
    let url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("Set TEST_DATABASE_URL for tests");
    let mut conn = sqlx::PgConnection::connect(&url)
        .await
        .expect("connect for insertion failed");
    let suffix = Uuid::new_v4().simple().to_string();
    let short = &suffix[..8];
    let mut username = format!("{}_{}", base, short);
    if username.len() > 50 {
        username.truncate(50);
    }
    let email = format!("{}@example.com", username);
    let password = "TestPassword123!".to_string();
    let hash = pokedex_rncp_backend::auth::hash_password(&password).expect("hash_password failed");

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, email, password)
        VALUES ($1, $2, $3)
        ON CONFLICT (username)
        DO UPDATE SET email = EXCLUDED.email, password = EXCLUDED.password
        RETURNING id
        "#,
    )
    .bind(&username)
    .bind(&email)
    .bind(&hash)
    .fetch_one(&mut conn)
    .await
    .expect("insert user failed");

    (id, username, email, password)
}
#[allow(dead_code)]
pub async fn delete_user(username: &str) {
    if let Ok(url) = std::env::var("TEST_DATABASE_URL")
        && let Ok(mut conn) = sqlx::PgConnection::connect(&url).await
    {
        let _ = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&mut conn)
            .await;
    }
}
#[allow(dead_code)]
pub async fn ensure_pokemon(name: &str, type1: &str) -> i32 {
    let pool = test_pool().await;
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO pokemon (name, type1)
        VALUES ($1, $2)
        ON CONFLICT (name) DO UPDATE SET type1 = EXCLUDED.type1
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(type1)
    .fetch_one(pool)
    .await
    .expect("insert/select pokemon failed");
    id
}
#[allow(dead_code)]
pub fn cookie_header(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}
