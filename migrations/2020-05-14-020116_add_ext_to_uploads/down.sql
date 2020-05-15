-- This file should undo anything in `up.sql`

ALTER TABLE uploads
DROP COLUMN file_ext;

ALTER TABLE uploads
DROP COLUMN IF EXISTS tag_string;

ALTER TABLE uploads
DROP COLUMN IF EXISTS tag_index;

DROP TRIGGER IF EXISTS trigger_uploads_on_tag_index_update ON uploads;

DROP INDEX IF EXISTS index_uploads_on_tags_index;