-- This file should undo anything in `up.sql`

ALTER TABLE uploads
DROP COLUMN IF EXISTS original_upload_date;