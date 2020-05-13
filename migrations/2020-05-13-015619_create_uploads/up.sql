-- Your SQL goes here

CREATE TABLE tags (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);

CREATE TABLE uploads (
  id SERIAL PRIMARY KEY,
  status SMALLINT NOT NULL DEFAULT 0,
  file_id TEXT NOT NULL UNIQUE,
  file_size BIGINT,
  file_name TEXT UNIQUE,
  md5_hash TEXT UNIQUE,
  uploader_user_id INTEGER REFERENCES users (id),
  source TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('uploads');

CREATE TABLE uploads_tags (
  id SERIAL PRIMARY KEY,
  tag_id INTEGER REFERENCES tags (id),
  upload_id INTEGER REFERENCES uploads (id),
  UNIQUE (tag_id, upload_id)
);