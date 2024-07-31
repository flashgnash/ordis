-- Your SQL goes here
ALTER TABLE users
ADD COLUMN stat_block_hash TEXT;

ALTER TABLE users
ADD COLUMN stat_block TEXT;
