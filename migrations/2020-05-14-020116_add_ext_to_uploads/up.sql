-- Your SQL goes here

ALTER TABLE uploads
ADD file_ext TEXT NOT NULL;

ALTER TABLE uploads
DROP CONSTRAINT uploads_file_name_key;