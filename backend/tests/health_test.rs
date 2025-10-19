use reqwest::StatusCode;

mod common;

#[tokio::test]
async fn get_root_retourne_bienvenue() {
    let (base, handle) = common::start_server().await;

    let res = reqwest::get(format!("{base}/")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.text().await.unwrap();
    assert_eq!(body, "Bienvenue sur le pokeRncp");

    handle.abort();
}

#[tokio::test]
async fn path_inconnu_retourne_404() {
    let (base, handle) = common::start_server().await;

    let res = reqwest::get(format!("{base}/does-not-exist")).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    handle.abort();
}