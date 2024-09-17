-- This file should undo anything in `up.sql`
-- Restore old columns
CREATE TEMPORARY TABLE users_restore(id TEXT PRIMARY KEY, username TEXT, count INTEGER);
INSERT INTO users_restore SELECT id, username, count FROM users;
DROP TABLE users;
CREATE TABLE users(id TEXT PRIMARY KEY, username TEXT, count INTEGER);
INSERT INTO users SELECT * FROM users_restore;
DROP TABLE users_restore;

