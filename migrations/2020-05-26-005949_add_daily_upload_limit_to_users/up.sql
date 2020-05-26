-- Your SQL goes here

ALTER TABLE users
ADD COLUMN daily_upload_limit INT NOT NULL DEFAULT 1;