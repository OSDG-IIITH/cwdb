-- key courses by code

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'courses' AND column_name = 'id') THEN
        RETURN;
    END IF;

    UPDATE courses c SET code = 'TMP.' || lpad(t.n::text, 3, '0')
    FROM (SELECT id, row_number() OVER (ORDER BY name) AS n FROM courses WHERE code = '') t
    WHERE c.id = t.id;

    UPDATE courses SET code = 'TMP.028' WHERE code = 'EC5.203' AND name = 'Communicaton and Controls in IoT';
    UPDATE courses SET code = 'TMP.029' WHERE code = 'HS2.303' AND name = 'Ethnography in praxis: An investigation into Field, Scales and Theory';

    ALTER TABLE resources ADD COLUMN course TEXT;
    UPDATE resources r SET course = c.code FROM courses c WHERE c.id = r.course_id;
    ALTER TABLE resources DROP COLUMN course_id;

    ALTER TABLE courses DROP COLUMN id;
    ALTER TABLE courses DROP COLUMN year;
    ALTER TABLE courses ALTER COLUMN code DROP DEFAULT;
    ALTER TABLE courses ADD PRIMARY KEY (code);
    ALTER TABLE courses RENAME COLUMN aliases TO shortnames;

    ALTER TABLE resources ADD CONSTRAINT resources_course_fkey
        FOREIGN KEY (course) REFERENCES courses(code) ON UPDATE CASCADE ON DELETE SET NULL;
    CREATE INDEX idx_resources_course ON resources(course);
END $$;
