use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PokemonWithCaught {
    pub id: i32,
    pub name: String,
    pub type1: String,
    pub type2: Option<String>,
    pub caught: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatchByNamePayload {
    pub name: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchParams {
    pub q: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PokemonDetail {
    pub id: i32,
    pub name: String,
    pub type1: String,
    pub type2: Option<String>,
    pub base_hp: Option<i32>,
    pub base_attack: Option<i32>,
    pub base_defense: Option<i32>,
    pub base_sp_attack: Option<i32>,
    pub base_sp_defense: Option<i32>,
    pub base_speed: Option<i32>,
    pub caught: bool,
}
