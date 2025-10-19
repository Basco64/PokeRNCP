use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPokemon {
    pub pokemon_id: i32,
    pub nickname: Option<String>,
    pub level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserPokemon {
    pub nickname: Option<String>,
    pub level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPokemon {
    pub id: i32,
    pub pokemon_id: i32,
    pub pokemon_name: String,
    pub type1: String,
    pub type2: Option<String>,
    pub nickname: Option<String>,
    pub level: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub captured_at: OffsetDateTime,
}
