# courseworkdb

## Prerequisites

- **Docker** (for running the database and search engine)
- **Rust** (for the backend)
- `bun` (for the frontend)

## Setup Instructions

### Environment Setup

Clone the repo and set up the environment variables:

```bash
cp .env.example .env
```

Update `.env` with your Microsoft Entra ID credentials ([Guide](https://gist.github.com/nuxshed/610509f2b7c1093ef6f6646d8c779707))

### Start Services

Start Postgres and Meilisearch:

```bash
docker-compose up -d
```

### Backend

Navigate to the backend directory:

```bash
cd backend
```

Install `sqlx-cli` if you dont already have it:

```bash
cargo install sqlx-cli
```

Run migrations:

```bash
sqlx migrate run
```

Run the backend server:

```bash
cargo run
```

The backend API will be available at `http://localhost:3000`.

### Frontend

Navigate to the frontend:

```bash
cd frontend
```

Install dependencies:

```bash
bun install
```

Start the development server:

```bash
bun dev
```

The frontend will be available at `http://localhost:5173`.

---

## Nix Setup :)

If you have Nix installed, you can use the flake ❄️

Enable the development shell using `nix develop`:

```bash
nix develop
```

_If you use `direnv`, simply run `direnv allow`._
