-- This file should undo anything in `up.sql`

ALTER TABLE upload_views
DROP COLUMN IF EXISTS viewed_at;