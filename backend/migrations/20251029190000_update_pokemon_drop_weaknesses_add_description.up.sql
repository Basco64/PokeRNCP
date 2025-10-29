-- Drop weaknesses column and add description to pokemon
ALTER TABLE pokemon
    DROP COLUMN IF EXISTS weaknesses,
    ADD COLUMN IF NOT EXISTS description TEXT;
