CREATE TABLE sources (
    id SERIAL PRIMARY KEY,
    owner VARCHAR(255) NOT NULL,
    repo VARCHAR(255) NOT NULL,
    branch VARCHAR(255) NOT NULL DEFAULT 'main',
    last_synced_at TIMESTAMPTZ,
    last_etag VARCHAR(255),
    poll_frequency INT NOT NULL DEFAULT 7,
    source_status VARCHAR(50) NOT NULL DEFAULT 'active',
    like_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID NOT NULL,
    UNIQUE(owner, repo, branch)
);

CREATE TABLE resources (
    id SERIAL PRIMARY KEY,
    source_id INT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    path_hash VARCHAR(64) NOT NULL,
    title VARCHAR(255) NOT NULL,
    tags TEXT[] DEFAULT '{}',
    like_count INT NOT NULL DEFAULT 0,
    download_url TEXT NOT NULL,
    type VARCHAR(50) NOT NULL DEFAULT '',
    sha VARCHAR(40) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source_id, path_hash)
);

CREATE TABLE likes (
    user_id UUID NOT NULL,
    resource_id INT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, resource_id)
);

CREATE INDEX idx_resources_source_id ON resources(source_id);
CREATE INDEX idx_resources_path_hash ON resources(path_hash);
CREATE INDEX idx_likes_resource_id ON likes(resource_id);

CREATE TABLE source_likes (
    user_id UUID NOT NULL,
    source_id INT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, source_id)
);

CREATE INDEX idx_source_likes_source_id ON source_likes(source_id);
