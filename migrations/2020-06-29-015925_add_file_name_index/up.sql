-- Your SQL goes here

CREATE INDEX index_uploads_on_file_name_index ON uploads USING gin (file_name gin_trgm_ops);