DROP TABLE IF EXISTS resources;

CREATE TABLE resources (
    id SERIAL PRIMARY KEY,
    source_id INT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    path_hash VARCHAR(64) NOT NULL,
    title VARCHAR(255) NOT NULL,
    tags TEXT[] DEFAULT '{}',
    vote_count INT NOT NULL DEFAULT 0,
    download_url TEXT NOT NULL,
    sha VARCHAR(40) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source_id, path_hash)
);

CREATE INDEX idx_resources_source_id ON resources(source_id);
CREATE INDEX idx_resources_path_hash ON resources(path_hash);
