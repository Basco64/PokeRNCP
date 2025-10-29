-- Migration: add_pokemon_image_and_number (UP)
-- Ajoute des colonnes pour stocker le numéro du Pokédex et l'URL de l'image

ALTER TABLE pokemon
    ADD COLUMN IF NOT EXISTS dex_no INTEGER,
    ADD COLUMN IF NOT EXISTS image_url TEXT;