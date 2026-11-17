-- profiles, my courses, paper progress and the almanac

CREATE TABLE IF NOT EXISTS profiles (
    userid    UUID        PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    autoterm  BOOLEAN     NOT NULL DEFAULT true,
    examfocus BOOLEAN     NOT NULL DEFAULT true,
    ug1       BOOLEAN     NOT NULL DEFAULT false,
    ug1since  TIMESTAMPTZ,
    prefs     JSONB       NOT NULL DEFAULT '{}'
);

-- archived is the manual flag, used while autoterm is off; with it on, the term of added decides
CREATE TABLE IF NOT EXISTS usercourses (
    userid   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course   TEXT        NOT NULL REFERENCES courses(code) ON UPDATE CASCADE ON DELETE CASCADE,
    added    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived BOOLEAN     NOT NULL DEFAULT false,
    PRIMARY KEY (userid, course)
);

CREATE TABLE IF NOT EXISTS progress (
    userid   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    resource INT         NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    seen     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    done     BOOLEAN     NOT NULL DEFAULT false,
    PRIMARY KEY (userid, resource)
);

-- evaluation windows per term, from the institute almanacs; published with the index
INSERT INTO settings (key, value) VALUES ('almanac', '{
    "S26": [
        { "exam": "quiz", "n": 1, "from": "2026-01-29", "to": "2026-01-31" },
        { "exam": "mid", "from": "2026-02-26", "to": "2026-03-02" },
        { "exam": "quiz", "n": 2, "from": "2026-03-30", "to": "2026-04-01" },
        { "exam": "end", "from": "2026-04-27", "to": "2026-05-05" }
    ],
    "M26": [
        { "exam": "quiz", "n": 1, "from": "2026-08-27", "to": "2026-08-29", "ug1": { "from": "2026-09-02", "to": "2026-09-05" } },
        { "exam": "mid", "from": "2026-09-21", "to": "2026-09-24" },
        { "exam": "quiz", "n": 2, "from": "2026-10-22", "to": "2026-10-24" },
        { "exam": "end", "from": "2026-11-23", "to": "2026-11-30" }
    ]
}'::jsonb) ON CONFLICT (key) DO NOTHING;
