-- Your SQL goes here

CREATE TABLE audit_log (
  id BIGSERIAL PRIMARY KEY,
  table_name TEXT NOT NULL,
  column_name TEXT NOT NULL,
  row_id INTEGER NOT NULL,
  changed_date TIMESTAMP NOT NULL DEFAULT current_timestamp,
  changed_by INTEGER REFERENCES users (id) NOT NULL,
  old_value TEXT NOT NULL,
  new_value TEXT NOT NULL
);

CREATE INDEX audit_log_idx ON audit_log (table_name, row_id);