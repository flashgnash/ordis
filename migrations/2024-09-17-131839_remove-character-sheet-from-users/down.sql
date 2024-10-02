-- This file should undo anything in `up.sql`
-- Restore old columns
CREATE TABLE users_new (
    -- List all the original columns, without the selected_character
    id INTEGER PRIMARY KEY,
    name TEXT,
    email TEXT
    -- Add other columns that existed in the original table "users"
);

INSERT INTO users_new (id, name, email)
SELECT id, name, email FROM users;

DROP TABLE users;

ALTER TABLE users_new RENAME TO users;
