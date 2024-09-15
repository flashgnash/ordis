CREATE TABLE new_characters (
    id INTEGER PRIMARY KEY,
    user_id TEXT,
    name TEXT,
    stat_block_hash TEXT,
    stat_block TEXT,
    stat_block_message_id TEXT,
    stat_block_channel_id TEXT
);

-- 2. Copy data from old table to new table
INSERT INTO new_characters (
    id,
    user_id,
    name,
    stat_block_hash,
    stat_block,
    stat_block_message_id,
    stat_block_channel_id
) SELECT id, user_id, name, stat_block_hash, stat_block, stat_block_message_id, stat_block_channel_id FROM characters;

-- 3. Drop old table
DROP TABLE characters;

-- 4. Rename new table to old table
ALTER TABLE new_characters RENAME TO characters;
