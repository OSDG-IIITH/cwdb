# cwdb

cwdb is a coursework discovery platform that helps students find useful course resources from previous years in one place.


## Prerequisites
Docker, Rust, `cargo`, `sqlx-cli` and `bun`


## Backend

The backend is written in Rust and uses Postgres for data storage.
Docker is used to run Postgres and Meilisearch.

Before starting the server, copy the backend env file:

```bash
cp backend/.env.example backend/.env
```

Then start the local services:

```bash
docker-compose up -d
```

Run database migrations:

```bash
cd backend
sqlx migrate run
```

If you have `ocas` set up and running, start the backend with:

```bash
cargo run
```

If you do not have `ocas` running, you can use mock authentication:

```bash
cargo run --features mock
```

The backend runs on `http://localhost:3000`.


## Frontend

Copy the frontend env file:

```bash
cp frontend/.env.example frontend/.env
```

If you want to use `mock` authentication, in `frontend/.env`, keep mock auth enabled and set a mock email:

```env
VITE_USE_MOCK_AUTH="true"
VITE_MOCK_EMAIL=
```

Then start the frontend:

```bash
cd frontend
bun install
bun dev
```

The frontend runs on `http://localhost:5173`.


## Nix Flake ❄️

If you use Nix, enter the development shell with:

```bash
nix develop
```

If you use `direnv`, run:

```bash
direnv allow
```
