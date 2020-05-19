-- Your SQL goes here

CREATE TABLE upload_views (
  id BIGSERIAL PRIMARY KEY,
  upload_id INTEGER REFERENCES uploads (id) NOT NULL
);