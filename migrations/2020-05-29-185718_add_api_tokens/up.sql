-- Your SQL goes here

CREATE TABLE api_tokens (
  id BIGSERIAL PRIMARY KEY,
  token TEXT NOT NULL,
  user_id INTEGER REFERENCES users (id) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('api_tokens');

CREATE UNIQUE INDEX api_tokens_token_idx ON api_tokens(token);