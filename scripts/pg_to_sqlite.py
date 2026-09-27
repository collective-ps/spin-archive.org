#!/usr/bin/env python3
"""Copy the spin-archive Postgres database into a fresh SQLite file.

Usage:
    python scripts/pg_to_sqlite.py --out spin-archive.db
        (reads the Postgres URL from --pg-url, $PG_URL, or PG_URL=... in .env.migrate)
    python scripts/pg_to_sqlite.py --csv-dir exports/ --out spin-archive.db
        (loads CSVs previously saved with --keep-csv instead of querying Postgres)

The SQLite schema comes from the app's migration, so the resulting file is
ready to be used by the app as-is (the migration is marked as already run).
"""

import argparse
import csv
import io
import os
import shutil
import sqlite3
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MIGRATION_DIR = ROOT / "migrations" / "2026-09-26-000000_create_sqlite_schema"
MIGRATION_VERSION = "20260926000000"
NULL_MARKER = r"\N"

# Columns per table, matching src/schema.rs (minus uploads.tag_index, which
# was a Postgres tsvector and no longer exists).
TABLES = {
    "users": [
        "id", "username", "password_hash", "email", "created_at", "updated_at",
        "role", "daily_upload_limit", "invited_by_user_id",
    ],
    "uploads": [
        "id", "status", "file_id", "file_size", "file_name", "md5_hash",
        "uploader_user_id", "source", "created_at", "updated_at", "file_ext",
        "tag_string", "video_encoding_key", "thumbnail_url", "video_url",
        "description", "original_upload_date",
    ],
    "upload_views": ["id", "upload_id", "viewed_at"],
    "audit_log": [
        "id", "table_name", "column_name", "row_id", "changed_date",
        "changed_by", "old_value", "new_value",
    ],
    "upload_comments": [
        "id", "upload_id", "user_id", "comment", "created_at", "updated_at",
    ],
    "tags": [
        "id", "name", "description", "created_at", "updated_at", "upload_count",
    ],
    "api_tokens": ["id", "token", "user_id", "created_at", "updated_at"],
    "forums": ["id", "title", "description", "order_key", "is_open"],
    "threads": [
        "id", "title", "forum_id", "author_id", "is_sticky", "is_open",
        "is_deleted", "created_at", "updated_at",
    ],
    "posts": [
        "id", "thread_id", "author_id", "content", "is_deleted", "created_at",
        "updated_at",
    ],
    "invitations": [
        "id", "code", "creator_id", "consumer_id", "created_at", "updated_at",
    ],
}

BOOLEAN_COLUMNS = {
    ("forums", "is_open"),
    ("threads", "is_sticky"),
    ("threads", "is_open"),
    ("threads", "is_deleted"),
    ("posts", "is_deleted"),
}


def fail(message):
    print(f"error: {message}", file=sys.stderr)
    sys.exit(1)


def read_env_migrate():
    path = ROOT / ".env.migrate"
    if not path.exists():
        return None
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line.startswith("PG_URL="):
            return line[len("PG_URL="):].strip().strip("'\"")
    return None


def find_psql():
    found = shutil.which("psql")
    if found:
        return found
    for version in ("18", "17", "16"):
        candidate = Path(rf"C:\Program Files\PostgreSQL\{version}\bin\psql.exe")
        if candidate.exists():
            return str(candidate)
    fail("psql not found on PATH or in C:\\Program Files\\PostgreSQL\\<version>\\bin")


def run_psql(psql, pg_url, *commands):
    args = [psql, "--no-psqlrc", "-X", "-v", "ON_ERROR_STOP=1", "-d", pg_url]
    for command in commands:
        args += ["-c", command]
    env = dict(os.environ, PGCLIENTENCODING="UTF8")
    result = subprocess.run(args, capture_output=True, env=env)
    if result.returncode != 0:
        fail(f"psql failed: {result.stderr.decode('utf-8', 'replace').strip()}")
    return result.stdout.decode("utf-8")


def export_table(psql, pg_url, table, columns):
    select = f"SELECT {', '.join(columns)} FROM {table} ORDER BY id"
    copy = rf"\copy ({select}) TO STDOUT WITH (FORMAT csv, HEADER, NULL '\N')"
    return run_psql(psql, pg_url, "SET datestyle TO ISO", copy)


def postgres_count(psql, pg_url, table):
    out = run_psql(psql, pg_url, f"SELECT count(*) FROM {table}")
    # Unaligned output isn't guaranteed; pick the first all-digit line.
    for line in out.splitlines():
        if line.strip().isdigit():
            return int(line.strip())
    fail(f"could not parse count for {table}: {out!r}")


def convert(table, column, value):
    if value == NULL_MARKER:
        return None
    if (table, column) in BOOLEAN_COLUMNS:
        if value in ("t", "true", "1"):
            return 1
        if value in ("f", "false", "0"):
            return 0
        fail(f"unexpected boolean {value!r} in {table}.{column}")
    return value


def load_table(db, table, columns, csv_text):
    reader = csv.reader(io.StringIO(csv_text, newline=""))
    header = next(reader, None)
    if header is None:
        fail(f"{table}: CSV is empty (missing header)")
    if header != columns:
        fail(f"{table}: CSV header {header} does not match expected {columns}")

    placeholders = ", ".join("?" for _ in columns)
    sql = f"INSERT INTO {table} ({', '.join(columns)}) VALUES ({placeholders})"
    count = 0
    for row in reader:
        if len(row) != len(columns):
            fail(f"{table}: row {count + 1} has {len(row)} fields, expected {len(columns)}")
        db.execute(sql, [convert(table, c, v) for c, v in zip(columns, row)])
        count += 1
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--pg-url", help="Postgres URL (default: $PG_URL or .env.migrate)")
    parser.add_argument("--csv-dir", help="load from CSVs in this dir instead of Postgres")
    parser.add_argument("--keep-csv", help="save the exported CSVs into this dir")
    parser.add_argument("--out", default="spin-archive.db", help="SQLite file to create")
    parser.add_argument("--force", action="store_true", help="overwrite --out if it exists")
    args = parser.parse_args()

    out = Path(args.out)
    if out.exists() or Path(f"{out}-wal").exists():
        if not args.force:
            fail(f"{out} already exists (use --force to overwrite)")
        for suffix in ("", "-wal", "-shm", "-journal"):
            Path(f"{out}{suffix}").unlink(missing_ok=True)

    psql = pg_url = None
    if args.csv_dir:
        csv_dir = Path(args.csv_dir)
        exports = {}
        for table in TABLES:
            path = csv_dir / f"{table}.csv"
            if not path.exists():
                fail(f"missing {path}")
            exports[table] = path.read_text(encoding="utf-8")
    else:
        pg_url = args.pg_url or os.environ.get("PG_URL") or read_env_migrate()
        if not pg_url:
            fail("no Postgres URL: pass --pg-url, set PG_URL, or add PG_URL=... to .env.migrate")
        psql = find_psql()
        exports = {}
        for table, columns in TABLES.items():
            print(f"exporting {table}...", flush=True)
            exports[table] = export_table(psql, pg_url, table, columns)
        if args.keep_csv:
            keep = Path(args.keep_csv)
            keep.mkdir(parents=True, exist_ok=True)
            for table, text in exports.items():
                (keep / f"{table}.csv").write_text(text, encoding="utf-8", newline="")

    db = sqlite3.connect(out, isolation_level=None)
    db.execute("PRAGMA foreign_keys = OFF")
    db.executescript((MIGRATION_DIR / "up.sql").read_text(encoding="utf-8"))
    db.execute(
        "CREATE TABLE IF NOT EXISTS __diesel_schema_migrations ("
        "version VARCHAR(50) PRIMARY KEY NOT NULL, "
        "run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)"
    )
    db.execute("INSERT INTO __diesel_schema_migrations (version) VALUES (?)", (MIGRATION_VERSION,))

    loaded = {}
    db.execute("BEGIN")
    try:
        for table, columns in TABLES.items():
            loaded[table] = load_table(db, table, columns, exports[table])
        db.execute("COMMIT")
    except BaseException:
        db.execute("ROLLBACK")
        raise

    problems = []

    fk_violations = db.execute("PRAGMA foreign_key_check").fetchall()
    for table, rowid, parent, _ in fk_violations:
        problems.append(f"foreign key violation: {table} rowid={rowid} -> {parent}")

    integrity = db.execute("PRAGMA integrity_check").fetchone()[0]
    if integrity != "ok":
        problems.append(f"integrity_check: {integrity}")

    print()
    print(f"{'table':<18}{'sqlite':>10}{'postgres':>10}")
    for table in TABLES:
        sqlite_count = db.execute(f"SELECT count(*) FROM {table}").fetchone()[0]
        if sqlite_count != loaded[table]:
            problems.append(f"{table}: loaded {loaded[table]} rows but table has {sqlite_count}")
        if pg_url:
            pg_count = postgres_count(psql, pg_url, table)
            if pg_count != sqlite_count:
                problems.append(f"{table}: postgres has {pg_count} rows, sqlite has {sqlite_count}")
            print(f"{table:<18}{sqlite_count:>10}{pg_count:>10}")
        else:
            print(f"{table:<18}{sqlite_count:>10}{'-':>10}")

    db.execute("VACUUM")
    db.close()

    print()
    if problems:
        for problem in problems:
            print(f"PROBLEM: {problem}", file=sys.stderr)
        sys.exit(1)
    print(f"OK: wrote {out}")


if __name__ == "__main__":
    main()
