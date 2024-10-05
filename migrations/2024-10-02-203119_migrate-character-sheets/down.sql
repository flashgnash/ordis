-- Revert by copying the stat block data from character back to users
UPDATE users
SET 
    stat_block = (SELECT stat_block FROM characters WHERE characters.user_id = users.id AND characters.id = 1),
    stat_block_hash = (SELECT stat_block_hash FROM characters WHERE characters.user_id = users.id AND characters.id = 1),
    stat_block_message_id = (SELECT stat_block_message_id FROM characters WHERE characters.user_id = users.id AND characters.id = 1),
    stat_block_channel_id = (SELECT stat_block_channel_id FROM characters WHERE characters.user_id = users.id AND characters.id = 1)
WHERE EXISTS (
    SELECT 1 
    FROM characters 
    WHERE characters.user_id = users.id 
    AND characters.id = 1
);

-- Optionally delete the character with id 1 after the data is restored
DELETE FROM characters WHERE id = 1;

-- Clear the selected_character field in users table
UPDATE users
SET selected_character = NULL
WHERE selected_character = 1;
