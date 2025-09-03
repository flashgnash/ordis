ALTER TABLE servers
ADD COLUMN default_roll_server TEXT;

ALTER TABLE characters
ADD COLUMN stat_block_server_id TEXT;

ALTER TABLE characters
ADD COLUMN roll_server_id TEXT;
