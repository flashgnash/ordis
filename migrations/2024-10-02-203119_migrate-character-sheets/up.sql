-- Insert a new character for each user where message_id and channel_id exist in users,
-- but no character exists for that user
INSERT INTO characters (id, user_id, name, stat_block_hash, stat_block, stat_block_message_id, stat_block_channel_id)
SELECT 1, id, NULL, stat_block_hash, stat_block, stat_block_message_id, stat_block_channel_id
FROM users
WHERE stat_block_message_id IS NOT NULL 
  AND stat_block_channel_id IS NOT NULL
  AND NOT EXISTS (
      SELECT 1 
      FROM characters 
      WHERE characters.user_id = users.id
  );

-- Update the selected_character in users with the id of the newly created character
UPDATE users
SET selected_character = (
    SELECT id FROM characters 
    WHERE characters.user_id = users.id
    ORDER BY id ASC LIMIT 1 -- Select the character we just created, by user_id
)
WHERE stat_block_message_id IS NOT NULL 
  AND stat_block_channel_id IS NOT NULL
  AND selected_character IS NULL
  AND EXISTS (
      SELECT 1 
      FROM characters 
      WHERE characters.user_id = users.id
  );
