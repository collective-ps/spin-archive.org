-- Your SQL goes here

ALTER TABLE tags
ADD COLUMN upload_count INT NOT NULL DEFAULT 0;