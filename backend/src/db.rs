use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

async fn connect_to_db(url: &str) -> Result<PgPool, sqlx::Error> {
    let db_pool = PgPoolOptions::new().max_connections(5).connect(url).await?;

    Ok(db_pool)
}

async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    println!("🔄 Exécution des migrations...");

    let migration_result = sqlx::migrate!("./migrations").run(pool).await;

    match migration_result {
        Ok(_) => println!("✅ Migrations exécutées avec succès!"),
        Err(e) => println!("⚠️ Erreur lors des migrations : {}", e),
    }

    Ok(())
}

pub async fn init_db() -> PgPool {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = connect_to_db(&url)
        .await
        .unwrap_or_else(|e| panic!("Echec connexion DB: {e}"));
    let _ = run_migrations(&pool).await;
    pool
}
