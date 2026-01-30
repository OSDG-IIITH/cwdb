CREATE TABLE meilisearch_settings (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL
);

INSERT INTO meilisearch_settings (key, value) VALUES ('synonyms', '{
    "exam": ["test", "midterm", "final", "quiz"],
    "lecture": ["slides", "presentation"],
    "cpro": ["computer programming"],
    "dass": ["design and analysis of software systems"],
    "cso": ["computer systems organization"],
    "osn": ["operating systems and networks"],
    "iss": ["introduction to software systems"],
}');
