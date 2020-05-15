-- Your SQL goes here

ALTER TABLE uploads
ADD file_ext TEXT NOT NULL;

ALTER TABLE uploads
DROP CONSTRAINT IF EXISTS uploads_file_name_key;

DROP TABLE IF EXISTS uploads_tags;
DROP TABLE IF EXISTS tags;

ALTER TABLE uploads
ADD tag_string TEXT NOT NULL DEFAULT '';

ALTER TABLE uploads
ADD tag_index tsvector;

CREATE TRIGGER trigger_uploads_on_tag_index_update
BEFORE INSERT OR UPDATE ON uploads
FOR EACH ROW EXECUTE PROCEDURE tsvector_update_trigger('tag_index', 'pg_catalog.english', 'tag_string');

CREATE INDEX index_uploads_on_tags_index ON uploads USING gin (tag_index);