CREATE TABLE sources (
    id SERIAL PRIMARY KEY,
    owner VARCHAR(255) NOT NULL,
    repo VARCHAR(255) NOT NULL,
    branch VARCHAR(255) NOT NULL DEFAULT 'main',
    last_synced_at TIMESTAMPTZ,
    last_etag VARCHAR(255),
    poll_frequency INT NOT NULL DEFAULT 7,
    source_status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(owner, repo, branch)
);
