use axum::http::StatusCode;

pub type ApiResult<T> = Result<T, (StatusCode, String)>;

pub fn to_500<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

pub fn unique_or_500(e: sqlx::Error) -> (StatusCode, String) {
    if let sqlx::Error::Database(db) = &e {
        if db.code().as_deref() == Some("23505") {
            return (StatusCode::CONFLICT, "User already exists.".into());
        }
    }
    to_500(e)
}

pub fn ok(msg: impl Into<String>) -> ApiResult<(StatusCode, String)> {
    Ok((StatusCode::OK, msg.into()))
}

pub fn created(msg: impl Into<String>) -> ApiResult<(StatusCode, String)> {
    Ok((StatusCode::CREATED, msg.into()))
}

pub fn not_found(msg: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, msg.into())
}

pub fn unauthorized(msg: impl Into<String>) -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, msg.into())
}