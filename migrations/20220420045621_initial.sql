-- Add migration script here

CREATE TABLE users (
  id INT PRIMARY KEY,
  username VARCHAR(255) NOT NULL UNIQUE,
  password_hash VARCHAR(255) NOT NULL,
  email VARCHAR(255) UNIQUE,
  role TINYINT NOT NULL DEFAULT 0,
  created_at DATETIME NOT NULL DEFAULT current_timestamp,
  updated_at DATETIME NOT NULL DEFAULT current_timestamp ON UPDATE current_timestamp
);

CREATE TABLE uploads (
  id INT PRIMARY KEY,
  status TINYINT NOT NULL DEFAULT 0,
  file_id TEXT,
  file_size BIGINT,
  fize_name TEXT,
  md5_hash TEXT,
  uploader_user_id INT,
  source TEXT,
  file_text TEXT NOT NULL,
  tag_string TEXT NOT NULL DEFAULT '',
  FULLTEXT search_idx (tag_string),
  video_encoding_key TEXT NOT NULL DEFAULT '',
  thumbnail_url TEXT,
  video_url TEXT,
  description TEXT NOT NULL DEFAULT '',
  original_upload_date DATE,

  created_at DATETIME NOT NULL DEFAULT current_timestamp,
  updated_at DATETIME NOT NULL DEFAULT current_timestamp ON UPDATE current_timestamp
);

CREATE TABLE tags (
  id INT PRIMARY KEY,
  name VARCHAR(255) NOT NULL UNIQUE,
  description TEXT NOT NULL,
  upload_count INT NOT NULL DEFAULT 0,
  created_at DATETIME NOT NULL DEFAULT current_timestamp,
  updated_at DATETIME NOT NULL DEFAULT current_timestamp ON UPDATE current_timestamp
);

CREATE TABLE upload_comments (
  id INT PRIMARY KEY,
  upload_id INT NOT NULL,
  user_id INT NOT NULL,
  comment TEXT NOT NULL DEFAULT '',
  created_at DATETIME NOT NULL DEFAULT current_timestamp,
  updated_at DATETIME NOT NULL DEFAULT current_timestamp ON UPDATE current_timestamp
);