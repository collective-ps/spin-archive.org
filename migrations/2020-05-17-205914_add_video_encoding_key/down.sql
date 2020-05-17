-- This file should undo anything in `up.sql`

ALTER TABLE uploads
DROP COLUMN IF EXISTS video_encoding_key;

ALTER TABLE uploads
DROP COLUMN IF EXISTS thumbnail_url;

ALTER TABLE uploads
DROP COLUMN IF EXISTS video_url;