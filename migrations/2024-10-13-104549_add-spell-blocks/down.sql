-- This file should undo anything in `up.sql`
ALTER TABLE characters
DROP COLUMN spell_block_channel_id;

ALTER TABLE characters
DROP COLUMN spell_block_message_id;


