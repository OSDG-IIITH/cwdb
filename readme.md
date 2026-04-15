# cwdb

cwdb is a coursework discovery platform that helps students find useful course resources from previous years in one place.


## Prerequisites

Docker, Rust, `cargo`, `sqlx-cli` and `bun`


Before starting the server, copy the backend


## Setup

Copy the env file:

```bash
cp .env.example .env
```

Start Postgres (port 5433) and Meilisearch (port 7700):

```bash
docker-compose up -d
```

Run database migrations:

```bash
cd backend && sqlx migrate run
```


## Backend

First, copy the `.env` file:
```
cd backend && cp .env.example .env
```

The backend supports two auth methods, selected at compile time:

| Feature | Auth | Command |
|---|---|---|
| `ocas` | [ocas](https://github.com/nuxshed/ocas) | `cargo run` |
| `cas` | iiit cas | `cargo run --no-default-features --features cas` |

Add the `mock` feature to either to use mock authentication when no auth server is accessible.

See `.env.example` for relevant environment variables.

The backend runs on `http://localhost:3000`.


## Frontend

Copy the frontend env file:

```bash
cp frontend/.env.example frontend/.env
```

If you're using mock auth, set a mock email in `frontend/.env`:

```env
VITE_USE_MOCK_AUTH="true"
VITE_MOCK_EMAIL=you@students.iiit.ac.in
```

Then start the frontend:

```bash
cd frontend
bun install
bun dev
```

The frontend runs on `http://localhost:5173`.


## Scripts

`cwdb` comes with scripts to seed and manage data.

**`courses.py`** merges `monsoon.json`, `spring.json`, and `extra.json` into SQL `INSERT` statements for seeding the `courses` table:

```bash
python3 backend/scripts/courses.py | psql $DATABASE_URL
```

**`sources.sql`** seeds the database with known sources:

```bash
psql $DATABASE_URL < backend/scripts/sources.sql
```

**`regclient.sh`** registers the app with an `ocas` instance and prints the client ID and secret to paste into your `.env`:

```bash
bash backend/scripts/regclient.sh
```


## Nix

If you use Nix, enter the development shell with:

```bash
nix develop
```

If you use `direnv`:

```bash
direnv allow
```
