use axum::http::StatusCode;

pub type ApiResult<T> = Result<T, (StatusCode, String)>;

fn strip_emoji_prefix(s: String) -> String {
    let without_emoji = s
        .strip_prefix("⚠️")
        .or_else(|| s.strip_prefix("❌"))
        .unwrap_or(&s);
    without_emoji.trim().to_string()
}

fn log_warn(msg: &str) {
    eprintln!("⚠️ {}", msg);
}

fn log_error(msg: &str) {
    eprintln!("❌ {}", msg);
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
