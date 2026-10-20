-- offerings, faculty and succession

CREATE TABLE IF NOT EXISTS offerings (
    id     SERIAL   PRIMARY KEY,
    course TEXT     NOT NULL REFERENCES courses(code) ON UPDATE CASCADE ON DELETE CASCADE,
    season CHAR(1)  NOT NULL CHECK (season IN ('M', 'S')),
    year   SMALLINT NOT NULL CHECK (year BETWEEN 2000 AND 2100),
    origin TEXT     NOT NULL DEFAULT 'local' CHECK (origin IN ('grapevine', 'local')),
    UNIQUE (course, season, year)
);

CREATE TABLE IF NOT EXISTS offeringfaculty (
    offering INT  NOT NULL REFERENCES offerings(id) ON DELETE CASCADE,
    name     TEXT NOT NULL,
    PRIMARY KEY (offering, name)
);

CREATE TABLE IF NOT EXISTS coursesuccession (
    predecessor TEXT NOT NULL REFERENCES courses(code) ON UPDATE CASCADE ON DELETE CASCADE,
    successor   TEXT NOT NULL REFERENCES courses(code) ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (predecessor, successor),
    CHECK (predecessor <> successor)
);

-- monsoon.json and spring.json describe M25 and S26, as in grapevine
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'courses' AND column_name = 'instructors') THEN
        RETURN;
    END IF;

    INSERT INTO offerings (course, season, year)
    SELECT c.code, t.season, t.year
    FROM courses c
    JOIN (VALUES ('monsoon', 'M', 2025), ('spring', 'S', 2026), ('both', 'M', 2025), ('both', 'S', 2026))
        AS t(semester, season, year) USING (semester)
    ON CONFLICT DO NOTHING;

    INSERT INTO offeringfaculty (offering, name)
    SELECT o.id, unnest(c.instructors) FROM offerings o JOIN courses c ON c.code = o.course
    ON CONFLICT DO NOTHING;

    ALTER TABLE courses DROP COLUMN instructors;
END $$;
