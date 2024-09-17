-- Your SQL goes here

-- Add new column
ALTER TABLE users ADD COLUMN selected_character INTEGER;

-- Remove old columns
CREATE TEMPORARY TABLE users_backup(id PRIMARY KEY NOT NULL, username, count, selected_character);
INSERT INTO users_backup SELECT id, username, count, selected_character FROM users;
DROP TABLE users;
CREATE TABLE users(id TEXT PRIMARY KEY NOT NULL, username TEXT, count INTEGER, selected_character INTEGER);
INSERT INTO users SELECT * FROM users_backup;
DROP TABLE users_backup;

