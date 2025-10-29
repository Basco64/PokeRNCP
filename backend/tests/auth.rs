use pokedex_rncp_backend as backend;

use reqwest::StatusCode;
use serde_json::json;
use sqlx::{Connection, Row};

mod common;
use common::{cookie_header, create_test_user, delete_user, start_server};

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
}

#[tokio::test]
async fn me_ok_avec_bearer() {
    let (uid, username, _email, _pwd) = create_test_user("basco").await;
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
    delete_user(&username).await;
}

#[tokio::test]
async fn refresh_token_requiert_bearer_et_definit_cookie() {
    let (uid, username, _email, _pwd) = create_test_user("basco_refresh").await;
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
    delete_user(&username).await;
}

#[tokio::test]
async fn reset_mot_de_passe_retourne_token_et_confirmation_mise_a_jour() {
    let old_password = "OldPassword123!";
    let new_password = "NewPassword123!";
    let (uid, username, email, _pwd) = create_test_user("basco_reset").await;
    let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL for tests");
    let mut conn = sqlx::PgConnection::connect(&url)
        .await
        .expect("connect for setup failed");
    let newhash = backend::auth::hash_password(old_password).unwrap();
    let _ = sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
        .bind(&newhash)
        .bind(uid)
        .execute(&mut conn)
        .await
        .unwrap();

    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/auth/request-password-reset", base))
        .json(&json!({ "email_or_username": email }))
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

    handle.abort();

    let url = std::env::var("TEST_DATABASE_URL").expect("Set TEST_DATABASE_URL for tests");
    let mut conn = sqlx::PgConnection::connect(&url)
        .await
        .expect("connect for verification failed");
    let row = sqlx::query(r#"SELECT password FROM users WHERE id = $1"#)
        .bind(uid)
        .fetch_one(&mut conn)
        .await
        .expect("user fetch failed");
    let password: String = row.get("password");
    assert!(backend::auth::verify_password(&password, new_password));
    delete_user(&username).await;
}

#[tokio::test]
async fn me_ok_avec_cookie_auth() {
    let (uid, username, _email, _pwd) = create_test_user("me_cookie").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let access = backend::auth::generate_access_token(uid).expect("access token");

    let res = client
        .get(format!("{}/api/auth/me", base))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(body["username"], username);

    handle.abort();
    delete_user(&username).await;
}

#[tokio::test]
async fn refresh_token_ok_avec_cookie_refresh() {
    let (uid, username, _email, _pwd) = create_test_user("refresh_cookie").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let refresh = backend::auth::generate_refresh_token(uid).expect("refresh token");

    let res = client
        .post(format!("{}/api/auth/refresh-token", base))
        .header(
            reqwest::header::COOKIE,
            cookie_header(&[("refresh", &refresh)]),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let set_cookie = res
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(set_cookie.contains("auth="));

    handle.abort();
    delete_user(&username).await;
}

#[tokio::test]
async fn logout_definit_cookies_expirants() {
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!("{}/api/auth/logout", base))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let set_cookie = res
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(set_cookie.contains("auth="));
    assert!(set_cookie.contains("Max-Age=0"));
    assert!(set_cookie.contains("refresh="));
    handle.abort();
}

#[tokio::test]
async fn tokens_invalides_donnent_401() {
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/api/auth/me", base))
        .bearer_auth("invalid.token")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = client
        .post(format!("{}/api/auth/refresh-token", base))
        .bearer_auth("invalid.token")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    handle.abort();
}
