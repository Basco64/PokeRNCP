use axum::http::StatusCode;
use uuid::Uuid;

pub type ApiResult<T> = Result<T, (StatusCode, String)>;

// a modif pour enlever mais por l'instant ca fonctionne avec ca
thread_local! {
    static TL_USER_ID: std::cell::RefCell<Option<Uuid>> = const { std::cell::RefCell::new(None) };
}

pub fn set_current_user(user: Option<Uuid>) {
    TL_USER_ID.with(|c| *c.borrow_mut() = user);
}

pub fn current_user() -> Option<Uuid> {
    TL_USER_ID.with(|c| *c.borrow())
}

fn fmt_user() -> String {
    match current_user() {
        Some(id) => format!("| user: {}", id),
        None => "| user non connecté".to_string(),
    }
}

fn strip_emoji_prefix(s: String) -> String {
    let without_emoji = s
        .strip_prefix("⚠️")
        .or_else(|| s.strip_prefix("❌"))
        .unwrap_or(&s);
    without_emoji.trim().to_string()
}

fn log_warn(msg: &str) {
    eprintln!("⚠️ {} {}", msg, fmt_user());
}

fn log_error(msg: &str) {
    eprintln!("❌ {} {}", msg, fmt_user());
}

pub fn to_500<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    let msg = e.to_string();
    log_error(&msg);
    (StatusCode::INTERNAL_SERVER_ERROR, strip_emoji_prefix(msg))
}

pub fn ok(msg: impl Into<String>) -> ApiResult<(StatusCode, String)> {
    Ok((StatusCode::OK, msg.into()))
}

pub fn created(msg: impl Into<String>) -> ApiResult<(StatusCode, String)> {
    Ok((StatusCode::CREATED, msg.into()))
}

pub fn not_found(msg: impl Into<String>) -> (StatusCode, String) {
    let msg = msg.into();
    log_warn(&msg);
    (StatusCode::NOT_FOUND, strip_emoji_prefix(msg))
}

pub fn unauthorized(msg: impl Into<String>) -> (StatusCode, String) {
    let msg = msg.into();
    log_warn(&msg);
    (StatusCode::UNAUTHORIZED, strip_emoji_prefix(msg))
}

pub fn conflict(msg: impl Into<String>) -> (StatusCode, String) {
    let msg = msg.into();
    log_warn(&msg);
    (StatusCode::CONFLICT, strip_emoji_prefix(msg))
}

pub fn internal_server_error(msg: impl Into<String>) -> (StatusCode, String) {
    let msg = msg.into();
    log_error(&msg);
    (StatusCode::INTERNAL_SERVER_ERROR, strip_emoji_prefix(msg))
}

pub async fn shutdown() {
    tokio::signal::ctrl_c().await.unwrap();
    println!("🛑 Arrét en cours...");
}

// Middleware simple pour nettoyer le contexte utilisateur au début de chaque requête
// (évite toute fuite d'un user précédent sur le même thread)
#[allow(clippy::unused_async)]
pub async fn clear_user_mw(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    set_current_user(None);
    next.run(req).await
}
