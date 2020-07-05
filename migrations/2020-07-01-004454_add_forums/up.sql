-- Your SQL goes here

CREATE TABLE forums (
  id BIGSERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  order_key INTEGER NOT NULL DEFAULT 0,
  is_open BOOLEAN NOT NULL DEFAULT true
);

SELECT diesel_manage_updated_at('forums');

CREATE INDEX index_forums_on_title_index ON forums USING gin (title gin_trgm_ops);
CREATE INDEX index_forums_on_description_index ON forums USING gin (description gin_trgm_ops);

CREATE TABLE threads (
  id BIGSERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  forum_id BIGINT REFERENCES forums (id) NOT NULL,
  author_id INT REFERENCES users (id) NOT NULL,
  is_sticky BOOLEAN NOT NULL DEFAULT false,
  is_open BOOLEAN NOT NULL DEFAULT true,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('threads');
CREATE INDEX index_threads_on_title_index ON threads USING gin (title gin_trgm_ops);

CREATE TABLE posts (
  id BIGSERIAL PRIMARY KEY,
  thread_id BIGINT REFERENCES threads (id) NOT NULL,
  author_id INT REFERENCES users (id) NOT NULL,
  content TEXT NOT NULL DEFAULT '',
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

SELECT diesel_manage_updated_at('posts');
CREATE INDEX index_posts_on_content_index ON posts USING gin (content gin_trgm_ops);