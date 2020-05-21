-- Your SQL goes here

BEGIN;

ALTER TABLE upload_views
DROP CONSTRAINT upload_views_upload_id_fkey;

ALTER TABLE upload_views
ADD CONSTRAINT upload_views_upload_id_fkey
FOREIGN KEY (upload_id)
REFERENCES uploads (id)
ON DELETE CASCADE;

COMMIT;