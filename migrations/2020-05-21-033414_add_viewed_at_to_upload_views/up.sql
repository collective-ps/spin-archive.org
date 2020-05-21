-- Your SQL goes here

ALTER TABLE upload_views
ADD COLUMN viewed_at TIMESTAMP NOT NULL DEFAULT current_timestamp;
