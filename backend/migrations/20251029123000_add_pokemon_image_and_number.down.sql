-- Migration: add_pokemon_image_and_number (DOWN)
-- Supprime les colonnes et la contrainte associée

ALTER TABLE pokemon DROP CONSTRAINT IF EXISTS pokemon_dex_no_unique;

ALTER TABLE pokemon
    DROP COLUMN IF EXISTS dex_no,
    DROP COLUMN IF EXISTS image_url;
