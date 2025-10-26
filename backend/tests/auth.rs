use pokedex_rncp_backend as backend;

use reqwest::StatusCode;
use serde_json::json;
use sqlx::{Connection, Row};
use uuid::Uuid;

mod common;
use common::{start_server, test_pool};

async fn insertion_test_user(username: &str, email: &str, password: &str) -> Uuid {
    let _ = test_pool().await;
    let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL for tests");
    let mut conn = sqlx::PgConnection::connect(&url)
        .await
        .expect("connect for insertion failed");
    let hash = backend::auth::hash_password(password).expect("hash_password failed");
    let row = sqlx::query(
        r#"
        INSERT INTO users (username, email, password)
        VALUES ($1, $2, $3)
        ON CONFLICT (username)
        DO UPDATE SET email = EXCLUDED.email, password = EXCLUDED.password
        RETURNING id
        "#,
    )
    .bind(username)
    .bind(email)
    .bind(hash)
    .fetch_one(&mut conn)
    .await
    .expect("insert user failed");

    row.get::<Uuid, _>("id")
}

async fn suppression_user(username: &str) {
    if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        if let Ok(mut conn) = sqlx::PgConnection::connect(&url).await {
            let _ = sqlx::query("DELETE FROM users WHERE username = $1")
                .bind(username)
                .execute(&mut conn)
                .await;
        }
    }
}

#[tokio::test]
async fn me_requiert_authentification() {
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/api/auth/me", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    handle.abort();
    suppression_user("basco").await;
}

#[tokio::test]
async fn me_ok_avec_bearer() {
    let uid = insertion_test_user("basco", "basco@example.com", "Password123!").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let access = backend::auth::generate_access_token(uid).expect("access token");

    let res = client
        .get(format!("{}/api/auth/me", base))
        .bearer_auth(access)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    handle.abort();
    suppression_user("basco").await;
}

#[tokio::test]
async fn refresh_token_requiert_bearer_et_definit_cookie() {
    let uid =
        insertion_test_user("basco_refresh", "basco_refresh@example.com", "Password123!").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let refresh = backend::auth::generate_refresh_token(uid).expect("refresh token");

    let res = client
        .post(format!("{}/api/auth/refresh-token", base))
        .bearer_auth(refresh)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let set_cookie = res
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        set_cookie.contains("auth="),
        "SET-COOKIE should contain auth=, got: {}",
        set_cookie
    );

    handle.abort();
    suppression_user("basco_refresh").await;
}

#[tokio::test]
async fn reset_mot_de_passe_retourne_token_et_confirmation_mise_a_jour() {
    let old_password = "OldPassword123!";
    let new_password = "NewPassword123!";
    let uid = insertion_test_user("basco_reset", "basco_reset@example.com", old_password).await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/auth/request-password-reset", base))
        .json(&json!({ "email_or_username": "basco_reset@example.com" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let token = backend::auth::generate_reset_token(uid).expect("reset token");
    let res = client
        .post(format!("{}/api/auth/confirm-password-reset", base))
        .json(&json!({ "token": token, "new_password": new_password }))
        .send()
        .await
        .unwrap();
    let status = res.status();
    if status != StatusCode::OK {
        let body = res.text().await.unwrap_or_default();
        panic!(
            "confirm-password-reset expected 200, got {} with body: {}",
            status, body
        );
    }

    // Stop the server before verifying to avoid any pool contention
    handle.abort();

    let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL for tests");
    let mut conn = sqlx::PgConnection::connect(&url)
        .await
        .expect("connect for verification failed");
    let row = sqlx::query(r#"SELECT password FROM users WHERE email = $1"#)
        .bind("basco_reset@example.com")
        .fetch_one(&mut conn)
        .await
        .expect("user fetch failed");
    let password: String = row.get("password");
    assert!(backend::auth::verify_password(&password, new_password));
    suppression_user("basco_reset").await;
}
