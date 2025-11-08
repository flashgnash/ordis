ALTER TABLE servers
DROP COLUMN default_roll_server;

ALTER TABLE characters
DROP COLUMN stat_block_server_id;

ALTER TABLE characters
DROP COLUMN roll_server_id;
