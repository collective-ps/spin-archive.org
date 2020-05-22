-- Your SQL goes here

CREATE TABLE upload_comments (
  id BIGSERIAL PRIMARY KEY,
  upload_id INTEGER REFERENCES uploads (id) ON DELETE CASCADE NOT NULL,
  user_id INTEGER REFERENCES users (id) ON DELETE CASCADE NOT NULL,
  comment TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('upload_comments');