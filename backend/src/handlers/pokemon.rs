use axum::{
    Json,
    extract::Query,
    extract::{Path, State},
};
use sqlx::PgPool;

use crate::auth::CurrentUser;
use crate::helpers::{ApiResult, created, not_found, to_500};
use crate::models::pokemon::{CatchByNamePayload, PokemonDetail, PokemonWithCaught, SearchParams};

pub async fn list_all(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
) -> ApiResult<Json<Vec<PokemonWithCaught>>> {
    let rows = sqlx::query_as::<_, PokemonWithCaught>(
        r#"
        SELECT
            p.id          AS id,
            p.name        AS name,
            p.type1       AS type1,
            p.type2       AS type2,
            p.dex_no      AS dex_no,
            p.image_url   AS image_url,
            EXISTS (
                SELECT 1 FROM user_pokemon up
                WHERE up.user_id = $1 AND up.pokemon_id = p.id
            )            AS caught
        FROM pokemon p
        ORDER BY p.id
        "#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(to_500)?;

    Ok(Json(rows))
}

pub async fn catch(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CatchByNamePayload>,
) -> ApiResult<(axum::http::StatusCode, String)> {
    let pokemon_id = sqlx::query_scalar::<_, i32>(r#"SELECT id FROM pokemon WHERE name = $1"#)
        .bind(&payload.name)
        .fetch_optional(&pool)
        .await
        .map_err(to_500)?
        .ok_or_else(|| not_found("Pokémon introuvable."))?;

    let _ = sqlx::query(
        r#"
        INSERT INTO user_pokemon (user_id, pokemon_id, nickname)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, pokemon_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(pokemon_id)
    .bind(payload.nickname)
    .execute(&pool)
    .await
    .map_err(to_500)?;

    created("Pokémon marqué comme capturé.")
}

pub async fn search_pokemons(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Query(params): Query<SearchParams>,
) -> ApiResult<Json<Vec<PokemonWithCaught>>> {
    let q = format!("{}%", params.q);
    let rows = sqlx::query_as::<_, PokemonWithCaught>(
        r#"
        SELECT
            p.id          AS id,
            p.name        AS name,
            p.type1       AS type1,
            p.type2       AS type2,
            p.dex_no      AS dex_no,
            p.image_url   AS image_url,
            EXISTS (
                SELECT 1 FROM user_pokemon up
                WHERE up.user_id = $1 AND up.pokemon_id = p.id
            )            AS caught
        FROM pokemon p
        WHERE p.name ILIKE $2
        ORDER BY p.name
        LIMIT 10
        "#,
    )
    .bind(user_id)
    .bind(q)
    .fetch_all(&pool)
    .await
    .map_err(to_500)?;
    Ok(Json(rows))
}

pub async fn get_pokemon_by_id(
    CurrentUser(user_id): CurrentUser,
    State(pool): State<PgPool>,
    Path(pokemon_id): Path<i32>,
) -> ApiResult<Json<PokemonDetail>> {
    let row = sqlx::query_as::<_, PokemonDetail>(
        r#"
        SELECT
            p.id                 AS id,
            p.name               AS name,
            p.type1              AS type1,
            p.type2              AS type2,
            p.dex_no             AS dex_no,
            p.image_url          AS image_url,
            p.height_m           AS height_m,
            p.weight_kg          AS weight_kg,
            p.description        AS description,
            p.base_hp            AS base_hp,
            p.base_attack        AS base_attack,
            p.base_defense       AS base_defense,
            p.base_sp_attack     AS base_sp_attack,
            p.base_sp_defense    AS base_sp_defense,
            p.base_speed         AS base_speed,
            EXISTS (
                SELECT 1 FROM user_pokemon up
                WHERE up.user_id = $1 AND up.pokemon_id = p.id
            )                   AS caught
        FROM pokemon p
        WHERE p.id = $2
        "#,
    )
    .bind(user_id)
    .bind(pokemon_id)
    .fetch_optional(&pool)
    .await
    .map_err(to_500)?;

    let Some(row) = row else {
        return Err(not_found("Pokémon introuvable."));
    };

    Ok(Json(row))
}
