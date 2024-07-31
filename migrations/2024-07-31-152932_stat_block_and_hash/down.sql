-- This file should undo anything in `up.sql`

--SQLite does not support dropping columns

-- Step 1: Create the new table without the columns to be dropped

CREATE TABLE users_new (
    id TEXT NOT NULL,
    username TEXT,
    count INTEGER DEFAULT 0,

    PRIMARY KEY(id)
);

-- Step 2: Copy data from the old table to the new table
INSERT INTO users_new (id, username,count)
SELECT id, username,count
FROM users;

-- Step 3: Drop the old table
DROP TABLE users;

-- Step 4: Rename the new table to the original table name
ALTER TABLE users_new RENAME TO users;
