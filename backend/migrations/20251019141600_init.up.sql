-- Add up migration script here
-- Requis pour gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ===== TABLE USERS =====
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===== TABLE POKEMON =====
CREATE TABLE IF NOT EXISTS pokemon (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    type1 VARCHAR(20) NOT NULL,
    type2 VARCHAR(20),
    base_hp INTEGER,
    base_attack INTEGER,
    base_defense INTEGER,
    base_sp_attack INTEGER,
    base_sp_defense INTEGER,
    base_speed INTEGER
);

-- ===== TABLE ASSOCIATIVE USER_POKEMON =====
CREATE TABLE IF NOT EXISTS user_pokemon (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pokemon_id INTEGER NOT NULL REFERENCES pokemon(id),
    nickname VARCHAR(50),
    discovered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, pokemon_id)
);

-- ===== INDEX =====
CREATE INDEX IF NOT EXISTS idx_user_pokemon_user_id ON user_pokemon(user_id);
CREATE INDEX IF NOT EXISTS idx_user_pokemon_pokemon_id ON user_pokemon(pokemon_id);