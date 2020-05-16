-- Your SQL goes here

ALTER TABLE uploads
ALTER COLUMN tag_index SET DEFAULT '';

ALTER TABLE uploads
ALTER COLUMN tag_index SET NOT NULL;