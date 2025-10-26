mod common;

#[tokio::test]
async fn la_base_de_donnees_repond() {
    let pool = common::test_pool().await;
    let one: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(one, 1);
}
