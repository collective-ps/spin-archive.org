-- This file should undo anything in `up.sql`

ALTER TABLE tags
DROP COLUMN IF EXISTS upload_count;