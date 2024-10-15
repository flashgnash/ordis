use std::fmt;

use crate::common::Context;
use crate::common::Error;

use crate::db::models::Character;

use super::stat_puller;

use poise::serenity_prelude::Message;

pub struct SpellSheet {
    pub original_message: Option<String>,
    pub jsonified_message: Option<String>,
    pub message_hash: Option<String>,
    pub changed: bool,
    pub character: Option<Character>,
}

impl fmt::Display for SpellSheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(json) = self.jsonified_message.as_ref() {
            return write!(f, "{json}");
        }
        if let Some(message) = self.original_message.as_ref() {
            return write!(f, "{message}");
        }

        write!(f, "No spell sheet found")
    }
}
impl super::stat_puller::CharacterSheetable for SpellSheet {
    fn new() -> Self {
        return Self {
            original_message: None,
            jsonified_message: None,
            message_hash: None,
            changed: false,
            character: None,
        };
    }

    fn update_character(&mut self) {
        let mut char = self.character.clone().unwrap_or(Character::new_empty());

        char.spell_block = Some(
            self.jsonified_message_mut()
                .clone()
                .expect("Character sheet should always generate jsonified message"),
        );
        char.spell_block_hash = self.message_hash.clone();

        self.character = Some(char);
    }

    fn get_changed(&self) -> bool {
        self.changed
    }
    fn set_changed(&mut self, value: bool) {
        self.changed = value;
    }
    fn set_character(&mut self, char: Character) {
        self.character = Some(char.clone());
    }
    fn get_character(&self) -> Option<Character> {
        return self.character.clone();
    }
    fn get_hash(&self) -> Option<String> {
        self.message_hash.clone()
    }
    fn set_hash(&mut self, hash: String) {
        self.message_hash = Some(hash);
    }
    fn jsonified_message_mut(&mut self) -> &mut Option<String> {
        &mut self.jsonified_message
    }
    fn original_message_mut(&mut self) -> &mut Option<String> {
        &mut self.original_message
    }

    fn get_previous_hash(character: &Character) -> Option<String> {
        return character.spell_block_hash.clone();
    }

    async fn get_sheet_message(ctx: &Context<'_>, character: &Character) -> Result<Message, Error> {
        if let (Some(channel_id_u64), Some(message_id_u64)) = (
            character.spell_block_channel_id.clone(),
            character.spell_block_message_id.clone(),
        ) {
            let channel_id = channel_id_u64.parse().expect("Invalid channel ID");
            let message_id = message_id_u64.parse().expect("Invalid message ID");

            let message = crate::common::fetch_message_poise(&ctx, channel_id, message_id).await?;

            return Ok(message);
        }

        Err(Box::new(stat_puller::StatPullerError::NoCharacterSheet))
    }

    const PROMPT: &'static str = r#"
    You are a spell list pulling program. 
    Following this prompt you will receive a block of spells and their costs.
    Use the following schema:    
    {

        "spells": {
            fireball": {
                "type": "single",
                "cost": -150,
                "cast_time": "1 turn"
            },
            "invisibility": {
                "type": "toggle"
                "cost": -50,
                "cast_time": "instant"
            },
            "regen": {
                "type": "toggle",
                "cost": 50,
                "cast_time": "1 turn"
            }
        }
    }    

    If there are missing values, interpret them as null
    For cast time, use the middle value that should look like '2 actions'
    If there are spaces in spell names, remove them, replacing them with underscores
    If you are expecting a value in a specific format but it is incorrect, instead set the value as 'ERROR - (explanation)'
    You should translate these spells into a json dictionary.
    All keys should be lower case and spell corrected. Respond with only valid json, anything else will break the program"                    
"#;
}
