-- Your SQL goes here
ALTER TABLE users
ADD COLUMN stat_block_message_id TEXT;

ALTER TABLE users
ADD COLUMN stat_block_channel_id TEXT;
