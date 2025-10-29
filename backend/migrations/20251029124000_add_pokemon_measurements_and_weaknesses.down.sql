-- Migration: add_pokemon_measurements_and_weaknesses (DOWN)
-- Supprime les colonnes ajoutées

ALTER TABLE pokemon
    DROP COLUMN IF EXISTS height_m,
    DROP COLUMN IF EXISTS weight_kg,
    DROP COLUMN IF EXISTS weaknesses;
