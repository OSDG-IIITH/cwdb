-- classification fields on resources

ALTER TABLE resources
    ADD COLUMN IF NOT EXISTS size         BIGINT,
    ADD COLUMN IF NOT EXISTS ext          TEXT,
    ADD COLUMN IF NOT EXISTS offering     INT REFERENCES offerings(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS kind         TEXT CHECK (kind IN ('exams', 'assignments', 'labs', 'slides', 'notes', 'references', 'projects', 'other')),
    ADD COLUMN IF NOT EXISTS examtype     TEXT CHECK (examtype IN ('quiz', 'mid', 'end', 'lab')),
    ADD COLUMN IF NOT EXISTS slidetype    TEXT CHECK (slidetype IN ('lec', 'tut')),
    ADD COLUMN IF NOT EXISTS n            INT,
    ADD COLUMN IF NOT EXISTS key          BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS keyfor       INT REFERENCES resources(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS unit         TEXT,
    ADD COLUMN IF NOT EXISTS hidden       BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS classifiedby TEXT CHECK (classifiedby IN ('rule', 'llm', 'manual')),
    ADD COLUMN IF NOT EXISTS confidence   REAL CHECK (confidence BETWEEN 0 AND 1),
    ADD COLUMN IF NOT EXISTS needsreview  BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS classifiedat TIMESTAMPTZ;

-- title becomes an admin display override; it used to hold the filename
ALTER TABLE resources ALTER COLUMN title DROP NOT NULL;
UPDATE resources SET title = NULL WHERE title = regexp_replace(file_path, '^.*/', '');

UPDATE resources SET ext = lower(substring(file_path FROM '\.([^./]+)$')) WHERE ext IS NULL;

UPDATE resources
SET kind = CASE type WHEN 'exam' THEN 'exams' ELSE 'slides' END,
    classifiedby = 'rule',
    classifiedat = NOW()
WHERE kind IS NULL AND classifiedby IS NULL AND type IN ('exam', 'slides');
