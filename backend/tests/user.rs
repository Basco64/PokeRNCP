use pokedex_rncp_backend as backend;

use reqwest::StatusCode;
use serde_json::json;
use sqlx::{Connection, Row};

mod common;
use common::{create_test_user, delete_user, start_server};

#[tokio::test]
async fn create_user_201_et_conflit_409() {
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let unique = uuid::Uuid::new_v4().to_string();
    let username = format!("user_{}", unique);
    let email = format!("{}@example.com", username);
    let res = client
        .post(format!("{}/api/users", base))
        .json(&json!({
            "username": username,
            "email": email,
            "password": "Password123!"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = client
        .post(format!("{}/api/users", base))
        .json(&json!({
            "username": username,
            "email": email,
            "password": "Password123!"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);

    handle.abort();
    delete_user(&username).await;
}

#[tokio::test]
async fn update_user_refuse_si_different() {
    let (a_id, a_username, _a_email, _a_pwd) = create_test_user("userA").await;
    let (b_id, b_username, _b_email, _b_pwd) = create_test_user("userB").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let access = backend::auth::generate_access_token(a_id).unwrap();
    let res = client
        .patch(format!("{}/api/users/{}", base, b_id))
        .bearer_auth(access)
        .json(&json!({"username": "hacker"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    handle.abort();
    delete_user(&a_username).await;
    delete_user(&b_username).await;
}

#[tokio::test]
async fn update_user_soi_meme_modifie_username_email_et_password() {
    let (uid, username, email, _pwd) = create_test_user("user_self").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();

    let access = backend::auth::generate_access_token(uid).unwrap();

    let new_username = format!("{}_new", username);
    let new_email = format!("new_{}", email);
    let res = client
        .patch(format!("{}/api/users/{}", base, uid))
        .bearer_auth(&access)
        .json(&json!({"username": new_username, "email": new_email}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .patch(format!("{}/api/users/{}", base, uid))
        .bearer_auth(&access)
        .json(&json!({"password": "AnotherPass123!"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .get(format!("{}/api/auth/me", base))
        .bearer_auth(&access)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let me = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(me["username"], new_username);
    assert_eq!(me["email"], new_email);

    let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL");
    let mut conn = sqlx::PgConnection::connect(&url).await.unwrap();
    let row = sqlx::query("SELECT password FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let hash: String = row.get("password");
    assert!(backend::auth::verify_password(&hash, "AnotherPass123!"));

    handle.abort();
    delete_user(me["username"].as_str().unwrap()).await;
}

#[tokio::test]
async fn delete_user_soi_meme() {
    let (uid, username, _email, _pwd) = create_test_user("user_del").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();

    let access = backend::auth::generate_access_token(uid).unwrap();

    let res = client
        .delete(format!("{}/api/users/{}", base, uid))
        .bearer_auth(&access)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = client
        .delete(format!("{}/api/users/{}", base, uid))
        .bearer_auth(&access)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    handle.abort();
    delete_user(&username).await;
}
