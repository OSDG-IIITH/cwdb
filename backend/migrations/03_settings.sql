CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL
);

INSERT INTO settings (key, value) VALUES ('synonyms', '{
    "exam": ["test", "midterm", "final", "quiz"],
    "lecture": ["slides", "presentation"],
    "cpro": ["computer programming"],
    "dass": ["design and analysis of software systems"],
    "cso": ["computer systems organization"],
    "osn": ["operating systems and networks"],
    "iss": ["introduction to software systems"]
}'::jsonb);

INSERT INTO settings (key, value) VALUES ('global_ignore_patterns', '[
    ".git",
    ".github",
    "node_modules",
    "__pycache__",
    ".venv",
    "target",
    "dist",
    "build",
    "*.exe",
    "README.md",
    "readme.md",
    "LICENSE",
    "CONTRIBUTING.md",
    "contributing.md",
    "CHANGELOG.md",
    "changelog.md",
    "CODE_OF_CONDUCT.md",
    "code_of_conduct.md",
    "SECURITY.md",
    "security.md"
]'::jsonb);
