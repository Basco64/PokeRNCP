use pokedex_rncp_backend as backend;

use reqwest::StatusCode;
use serde_json::json;

mod common;
use common::{cookie_header, create_test_user, delete_user, start_server};

#[tokio::test]
async fn list_requiert_auth() {
    let (base, handle) = start_server().await;
    let res = reqwest::get(format!("{}/api/pokemons", base))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    handle.abort();
}

#[tokio::test]
async fn list_search_catch_get() {
    let (uid, username, _email, _pwd) = create_test_user("poke_user").await;
    let (base, handle) = start_server().await;
    let client = reqwest::Client::new();
    let access = backend::auth::generate_access_token(uid).unwrap();
    let res = client
        .get(format!("{}/api/pokemons", base))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list = res.json::<serde_json::Value>().await.unwrap();
    assert!(list.as_array().map(|a| !a.is_empty()).unwrap_or(false));

    let res = client
        .get(format!("{}/api/pokemons/search?q=Pi", base))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let results = res.json::<serde_json::Value>().await.unwrap();
    let names = results
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert!(names.iter().any(|n| n == "Pikachu"));

    let res = client
        .post(format!("{}/api/pokemons/catch", base))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .json(&json!({"name": "Pikachu"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let res = client
        .get(format!("{}/api/pokemons/search?q=Pikachu", base))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let search = res.json::<serde_json::Value>().await.unwrap();
    let pid1 = search[0]["id"].as_i64().unwrap() as i32;

    let res = client
        .get(format!("{}/api/pokemons/{}", base, pid1))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let detail = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(detail["id"].as_i64().unwrap() as i32, pid1);
    assert_eq!(detail["name"], "Pikachu");
    assert_eq!(detail["caught"], true);
    // new fields from seed
    assert_eq!(detail["dex_no"].as_i64().unwrap() as i32, 25);
    assert!(detail["image_url"].as_str().unwrap().contains("025.png"));
    let h = detail["height_m"].as_f64().unwrap();
    assert!((h - 0.41).abs() < 1e-6);
    let w = detail["weight_kg"].as_f64().unwrap();
    assert!((w - 6.0).abs() < 1e-6);
    let weaks = detail["weaknesses"].as_array().unwrap();
    assert_eq!(weaks, &vec![serde_json::Value::String("Ground".into())]);

    let res = client
        .get(format!("{}/api/pokemons/search?q=Pidgey", base))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let search2 = res.json::<serde_json::Value>().await.unwrap();
    let pid2 = search2[0]["id"].as_i64().unwrap() as i32;
    let res = client
        .get(format!("{}/api/pokemons/{}", base, pid2))
        .header(reqwest::header::COOKIE, cookie_header(&[("auth", &access)]))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let detail2 = res.json::<serde_json::Value>().await.unwrap();
    assert_eq!(detail2["caught"], false);

    handle.abort();
    delete_user(&username).await;
}
