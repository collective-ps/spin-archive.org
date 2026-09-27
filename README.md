# spin-archive

[spin-archive.org](https://spin-archive.org) is an internet archive project dedicated to preserving the history of pen spinning.

## Deploying (Fly.io)

The app runs as a single Fly Machine with its SQLite database on a volume
mounted at `/data` (see `fly.toml`). The Machine stops when idle and starts on
the next request.

```sh
fly volumes create spin_archive_data --region sea --size 1   # once
fly secrets set ROCKET_SECRET_KEY=$(openssl rand -base64 32) \
  AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=... COCONUT_API_KEY=... \
  DISCORD_WEBHOOK_URL=... DISCORD_CONTRIBUTOR_WEBHOOK_URL=...
fly deploy
```

`ROCKET_SECRET_KEY` must stay stable, or every login is invalidated whenever
the Machine restarts. Migrations run automatically on startup.

## Migrating data from Postgres

`scripts/pg_to_sqlite.py` copies the old Postgres database into a new SQLite
file using the schema in `migrations/`. It needs Python 3 and `psql`.

```sh
echo 'PG_URL=postgresql://user:pass@host:25060/db?sslmode=require' > .env.migrate
python scripts/pg_to_sqlite.py --out spin-archive.db --keep-csv exports/
```

It checks foreign keys and integrity, compares row counts with Postgres, and
exits non-zero if anything doesn't match. To upload the result:

```sh
fly ssh sftp shell   # put spin-archive.db /data/spin-archive.db
fly machine restart
```
