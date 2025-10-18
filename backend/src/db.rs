use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

pub async fn init_db() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap_or_else(|e| panic!("Echec connectin DB: {e}"))
}
