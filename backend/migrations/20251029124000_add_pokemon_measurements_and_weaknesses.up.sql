-- Migration: add_pokemon_measurements_and_weaknesses (UP)
-- Ajoute taille/poids (en unités SI) et faiblesses

ALTER TABLE pokemon
    ADD COLUMN IF NOT EXISTS height_m DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS weight_kg DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS weaknesses TEXT[];
