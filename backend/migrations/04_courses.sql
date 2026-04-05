-- courses registry and resource tagging

CREATE TABLE courses (
    id          UUID   PRIMARY KEY DEFAULT gen_random_uuid(),
    code        TEXT   NOT NULL DEFAULT '',
    name        TEXT   NOT NULL UNIQUE,
    aliases     TEXT[] NOT NULL DEFAULT '{}',
    instructors TEXT[] NOT NULL DEFAULT '{}',
    semester    TEXT   NOT NULL DEFAULT '',
    year        INT
);

ALTER TABLE resources ADD COLUMN course_id UUID REFERENCES courses(id);
CREATE INDEX idx_resources_course_id ON resources(course_id);

CREATE TABLE coursepins (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, course_id)
);
