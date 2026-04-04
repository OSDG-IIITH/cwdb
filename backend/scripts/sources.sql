INSERT INTO users (id, email)
VALUES ('00000000-0000-0000-0000-000000000001', 'admin@seed')
ON CONFLICT (id) DO NOTHING;

INSERT INTO sources (owner, repo, branch, created_by) VALUES ('arghyawning', 'my-iiith', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
INSERT INTO sources (owner, repo, branch, created_by) VALUES ('Fane1824', 'MDL-lecs', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
INSERT INTO sources (owner, repo, branch, created_by) VALUES ('Fane1824', 'IIIT-Coursework', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
INSERT INTO sources (owner, repo, branch, created_by) VALUES ('zyx7k', 'iiith-pyqs', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
INSERT INTO sources (owner, repo, branch, created_by) VALUES ('sanyamseac', 'exams2k24', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
INSERT INTO sources (owner, repo, branch, created_by) VALUES ('amoiba42', 'course-material-2k23', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
INSERT INTO sources (owner, repo, branch, created_by) VALUES ('shahiam', 'IIITH-CourseWork', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
INSERT INTO sources (owner, repo, branch, created_by) VALUES ('potatopudding-SSKS', 'Famous-Last-Words', 'main', '00000000-0000-0000-0000-000000000001') ON CONFLICT (owner, repo, branch) DO NOTHING;
