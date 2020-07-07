-- Your SQL goes here

CREATE TABLE invitations (
  id BIGSERIAL PRIMARY KEY,
  code TEXT NOT NULL,
  creator_id INT REFERENCES users (id) NOT NULL,
  consumer_id INT REFERENCES users (id),
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('invitations');

ALTER TABLE users
ADD invited_by_user_id INT REFERENCES users (id);

CREATE UNIQUE INDEX invitations_code_idx on invitations(code);