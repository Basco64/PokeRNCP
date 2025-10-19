-- Add down migration script here
-- pour rollback
DROP INDEX IF EXISTS idx_user_pokemon_pokemon_id;
DROP INDEX IF EXISTS idx_user_pokemon_user_id;

DROP TABLE IF EXISTS user_pokemon;
DROP TABLE IF EXISTS pokemon;
DROP TABLE IF EXISTS users;