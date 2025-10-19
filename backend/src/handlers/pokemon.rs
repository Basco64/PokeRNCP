use axum::{
    Json,
    extract::{Path, State},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::helpers::{ApiResult, created, not_found, ok, to_500};
use crate::models::pokemon::{CreateUserPokemon, UpdateUserPokemon, UserPokemon};

pub async fn list_pokemons(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<Json<Vec<UserPokemon>>> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    let rows = sqlx::query_as!(
        UserPokemon,
        r#"
        SELECT
            up.id                         AS "id!",
            up.pokemon_id                 AS "pokemon_id!",
            p.name                        AS "pokemon_name!",
            p.type1                       AS "type1!",
            p.type2                       AS "type2?",
            up.nickname                   AS "nickname?", 
            COALESCE(up.level, 1)         AS "level!: i32",      
            up.captured_at                AS "captured_at!: time::OffsetDateTime"
        FROM user_pokemon up
        JOIN pokemon p ON p.id = up.pokemon_id
        WHERE up.user_id = $1
        ORDER BY up.captured_at DESC
        "#,
        user_id
    )
    .fetch_all(&pool)
    .await
    .map_err(to_500)?;

    Ok(Json(rows))
}

pub async fn get_pokemon(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path((user_id, capture_id)): Path<(Uuid, i32)>,
) -> ApiResult<Json<UserPokemon>> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    let row = sqlx::query_as!(
        UserPokemon,
        r#"
        SELECT
            up.id                         AS "id!",
            up.pokemon_id                 AS "pokemon_id!",
            p.name                        AS "pokemon_name!",
            p.type1                       AS "type1!",
            p.type2                       AS "type2?",
            up.nickname                   AS "nickname?",
            COALESCE(up.level, 1)         AS "level!: i32",
            up.captured_at                AS "captured_at!: time::OffsetDateTime"
        FROM user_pokemon up
        JOIN pokemon p ON p.id = up.pokemon_id
        WHERE up.user_id = $1 AND up.id = $2
        "#,
        user_id,
        capture_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(to_500)?;

    let Some(row) = row else {
        return Err(not_found("Pokémon introuvable."));
    };

    Ok(Json(row))
}

pub async fn create_pokemon(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<CreateUserPokemon>,
) -> ApiResult<(axum::http::StatusCode, String)> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    let capture_id: i32 = sqlx::query_scalar!(
        r#"
        INSERT INTO user_pokemon (user_id, pokemon_id, nickname, level)
        VALUES ($1, $2, $3, COALESCE($4, 1))
        RETURNING id
        "#,
        user_id,
        payload.pokemon_id,
        payload.nickname,
        payload.level
    )
    .fetch_one(&pool)
    .await
    .map_err(to_500)?;

    created(&format!("Pokémon capturé (capture_id={capture_id})."))
}

pub async fn update_pokemon(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path((user_id, capture_id)): Path<(Uuid, i32)>,
    Json(payload): Json<UpdateUserPokemon>,
) -> ApiResult<(axum::http::StatusCode, String)> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    // COALESCE garde l’ancienne valeur si None (et évite NULL sur un champ NOT NULL)
    let res = sqlx::query!(
        r#"
        UPDATE user_pokemon
        SET
            nickname = COALESCE($3, nickname),
            level    = COALESCE($4, level)
        WHERE user_id = $1 AND id = $2
        "#,
        user_id,
        capture_id,
        payload.nickname,
        payload.level
    )
    .execute(&pool)
    .await
    .map_err(to_500)?;

    if res.rows_affected() == 0 {
        return Err(not_found("Pokémon introuvable."));
    }

    ok("Pokémon mis à jour.")
}

pub async fn delete_pokemon(
    CurrentUser(current_user): CurrentUser,
    State(pool): State<PgPool>,
    Path((user_id, capture_id)): Path<(Uuid, i32)>,
) -> ApiResult<(axum::http::StatusCode, String)> {
    if current_user != user_id {
        return Err(crate::helpers::unauthorized("ACCESS DENIED"));
    }
    let res = sqlx::query!(
        "DELETE FROM user_pokemon WHERE user_id = $1 AND id = $2",
        user_id,
        capture_id
    )
    .execute(&pool)
    .await
    .map_err(to_500)?;

    if res.rows_affected() == 0 {
        return Err(not_found("Pokémon introuvable."));
    }

    ok("Pokémon supprimé.")
}
