-- Tracks which encoder Machine is handling an upload. Uploads in Processing with
-- no machine are queued; the dispatcher starts a Machine for them when a slot frees up.
ALTER TABLE uploads ADD COLUMN encoder_machine_id TEXT;
ALTER TABLE uploads ADD COLUMN encoding_started_at TIMESTAMP;

-- Uploads left in Processing by the old Coconut integration are not re-queued.
UPDATE uploads SET encoder_machine_id = 'legacy', encoding_started_at = updated_at WHERE status = 1;

CREATE INDEX uploads_encoding_queue_idx ON uploads (status, encoder_machine_id);
