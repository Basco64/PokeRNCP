-- Revert: remove description and re-add weaknesses
ALTER TABLE pokemon
    DROP COLUMN IF EXISTS description,
    ADD COLUMN IF NOT EXISTS weaknesses TEXT[];
