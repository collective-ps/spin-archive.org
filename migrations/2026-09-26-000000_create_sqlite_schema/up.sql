-- SQLite schema, ported from the original Postgres migrations.
--
-- Timestamps are stored as text in `YYYY-MM-DD HH:MM:SS.SSS` form (UTC), which
-- Diesel reads as `NaiveDateTime` and which sorts correctly as text.

CREATE TABLE users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  email TEXT UNIQUE,
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  role SMALLINT NOT NULL DEFAULT 0,
  daily_upload_limit INTEGER NOT NULL DEFAULT 1,
  invited_by_user_id INTEGER REFERENCES users (id)
);

CREATE TABLE uploads (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  status SMALLINT NOT NULL DEFAULT 0,
  file_id TEXT NOT NULL UNIQUE,
  file_size BIGINT,
  file_name TEXT,
  md5_hash TEXT,
  uploader_user_id INTEGER REFERENCES users (id),
  source TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  file_ext TEXT NOT NULL,
  tag_string TEXT NOT NULL DEFAULT '',
  video_encoding_key TEXT NOT NULL,
  thumbnail_url TEXT,
  video_url TEXT,
  description TEXT NOT NULL DEFAULT '',
  original_upload_date DATE
);

CREATE INDEX uploads_status_created_at_idx ON uploads (status, created_at);
CREATE INDEX uploads_uploader_user_id_idx ON uploads (uploader_user_id);
CREATE INDEX uploads_md5_hash_idx ON uploads (md5_hash);
CREATE INDEX uploads_video_encoding_key_idx ON uploads (video_encoding_key);
CREATE INDEX uploads_source_idx ON uploads (source);

CREATE TABLE upload_views (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  upload_id INTEGER NOT NULL REFERENCES uploads (id) ON DELETE CASCADE,
  viewed_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now'))
);

CREATE INDEX upload_views_upload_id_idx ON upload_views (upload_id);

CREATE TABLE audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  table_name TEXT NOT NULL,
  column_name TEXT NOT NULL,
  row_id INTEGER NOT NULL,
  changed_date TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  changed_by INTEGER NOT NULL REFERENCES users (id),
  old_value TEXT NOT NULL,
  new_value TEXT NOT NULL
);

CREATE INDEX audit_log_idx ON audit_log (table_name, row_id);
CREATE INDEX audit_log_changed_date_idx ON audit_log (changed_date);

CREATE TABLE upload_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  upload_id INTEGER NOT NULL REFERENCES uploads (id) ON DELETE CASCADE,
  user_id INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  comment TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now'))
);

CREATE INDEX upload_comments_upload_id_idx ON upload_comments (upload_id);
CREATE INDEX upload_comments_user_id_idx ON upload_comments (user_id);
CREATE INDEX upload_comments_created_at_idx ON upload_comments (created_at);

CREATE TABLE tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  upload_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE api_tokens (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  token TEXT NOT NULL UNIQUE,
  user_id INTEGER NOT NULL REFERENCES users (id),
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now'))
);

CREATE TABLE forums (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  order_key INTEGER NOT NULL DEFAULT 0,
  is_open BOOLEAN NOT NULL DEFAULT 1
);

CREATE TABLE threads (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  forum_id BIGINT NOT NULL REFERENCES forums (id),
  author_id INTEGER NOT NULL REFERENCES users (id),
  is_sticky BOOLEAN NOT NULL DEFAULT 0,
  is_open BOOLEAN NOT NULL DEFAULT 1,
  is_deleted BOOLEAN NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now'))
);

CREATE TABLE posts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  thread_id BIGINT NOT NULL REFERENCES threads (id),
  author_id INTEGER NOT NULL REFERENCES users (id),
  content TEXT NOT NULL DEFAULT '',
  is_deleted BOOLEAN NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now'))
);

CREATE TABLE invitations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  code TEXT NOT NULL UNIQUE,
  creator_id INTEGER NOT NULL REFERENCES users (id),
  consumer_id INTEGER REFERENCES users (id),
  created_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now')),
  updated_at TIMESTAMP NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%f', 'now'))
);

-- Replaces Diesel's Postgres `diesel_manage_updated_at` triggers: bump
-- `updated_at` on any update that didn't set it explicitly.
CREATE TRIGGER users_set_updated_at AFTER UPDATE ON users FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE users SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER uploads_set_updated_at AFTER UPDATE ON uploads FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE uploads SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER upload_comments_set_updated_at AFTER UPDATE ON upload_comments FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE upload_comments SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER tags_set_updated_at AFTER UPDATE ON tags FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE tags SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER api_tokens_set_updated_at AFTER UPDATE ON api_tokens FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE api_tokens SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER threads_set_updated_at AFTER UPDATE ON threads FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE threads SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER posts_set_updated_at AFTER UPDATE ON posts FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE posts SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;

CREATE TRIGGER invitations_set_updated_at AFTER UPDATE ON invitations FOR EACH ROW
WHEN NEW.updated_at IS OLD.updated_at
BEGIN
  UPDATE invitations SET updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = NEW.id;
END;
