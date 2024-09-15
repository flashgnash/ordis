-- 1. Create old table structure with id as TEXT
CREATE TABLE old_characters (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    name TEXT,
    stat_block_hash TEXT,
    stat_block TEXT,
    stat_block_message_id TEXT,
    stat_block_channel_id TEXT
);

-- 2. Copy data from current table to old table
INSERT INTO old_characters (
    id,
    user_id,
    name,
    stat_block_hash,
    stat_block,
    stat_block_message_id,
    stat_block_channel_id
) SELECT id, user_id, name, stat_block_hash, stat_block, stat_block_message_id, stat_block_channel_id FROM characters;

-- 3. Drop current table
DROP TABLE characters;

-- 4. Rename old table to current table name
ALTER TABLE old_characters RENAME TO characters;
