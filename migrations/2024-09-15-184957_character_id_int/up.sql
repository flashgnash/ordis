DROP TABLE IF EXISTS new_characters;

CREATE TABLE new_characters (
    id INTEGER PRIMARY KEY,
    user_id TEXT,
    name TEXT,
    stat_block_hash TEXT,
    stat_block TEXT,
    stat_block_message_id TEXT,
    stat_block_channel_id TEXT
);

INSERT INTO new_characters (
    id,
    user_id,
    name,
    stat_block_hash,
    stat_block,
    stat_block_message_id,
    stat_block_channel_id
)
SELECT
    id::integer,
    user_id,
    name,
    stat_block_hash,
    stat_block,
    stat_block_message_id,
    stat_block_channel_id
FROM characters;

DROP TABLE characters;

ALTER TABLE new_characters RENAME TO characters;
