-- This file should undo anything in `up.sql`
ALTER TABLE characters
DROP COLUMN stat_block_hash;

ALTER TABLE characters
DROP COLUMN stat_block;

ALTER TABLE characters
DROP COLUMN stat_block_message_id;

ALTER TABLE characters
DROP COLUMN stat_block_channel_id;
