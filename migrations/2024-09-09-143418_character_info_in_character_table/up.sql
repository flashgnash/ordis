-- Your SQL goes here
ALTER TABLE characters
ADD COLUMN stat_block_hash TEXT;

ALTER TABLE characters
ADD COLUMN stat_block TEXT;

ALTER TABLE characters
ADD COLUMN stat_block_message_id TEXT;

ALTER TABLE characters
ADD COLUMN stat_block_channel_id TEXT;
