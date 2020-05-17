-- Your SQL goes here

ALTER TABLE uploads
ADD video_encoding_key TEXT;

UPDATE uploads SET video_encoding_key = '';

ALTER TABLE uploads
ALTER COLUMN video_encoding_key SET NOT NULL;

ALTER TABLE uploads
ADD thumbnail_url TEXT;

ALTER TABLE uploads
ADD video_url TEXT;