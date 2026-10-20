-- drop pins, likes and tags

DROP TABLE IF EXISTS coursepins;
DROP TABLE IF EXISTS likes;
DROP TABLE IF EXISTS source_likes;

ALTER TABLE resources
    DROP COLUMN IF EXISTS tags,
    DROP COLUMN IF EXISTS like_count;

ALTER TABLE sources DROP COLUMN IF EXISTS like_count;
