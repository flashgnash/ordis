-- This file should undo anything in `up.sql`
CREATE TABLE users_new (
    id TEXT NOT NULL,
    username TEXT,
    count INTEGER DEFAULT 0,
    stat_block TEXT,
    stat_block_hash TEXT,

    PRIMARY KEY(id)
);

-- Step 2: Copy data from the old table to the new table
INSERT INTO users_new (id, username,count,stat_block,stat_block_hash)
SELECT id, username,count,stat_block,stat_block_hash
FROM users;

-- Step 3: Drop the old table
DROP TABLE users;

-- Step 4: Rename the new table to the original table name
ALTER TABLE users_new RENAME TO users;
