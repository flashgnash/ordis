use std::fmt;

use crate::common::Context;
use crate::common::Error;

use crate::db::models::Character;

use super::stat_puller;
use super::stat_puller::SheetInfo;

use poise::serenity_prelude::Message;

pub struct SpellSheet {
    pub sheet_info: SheetInfo,
}

impl fmt::Display for SpellSheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(json) = self.sheet_info.jsonified_message.as_ref() {
            return write!(f, "{json}");
        }
        if let Some(message) = self.sheet_info.original_message.as_ref() {
            return write!(f, "{message}");
        }

        write!(f, "No spell sheet found")
    }
}

impl super::stat_puller::CharacterSheetable for SpellSheet {
    fn new() -> Self {
        return Self {
            sheet_info: SheetInfo {
                original_message: None,
                jsonified_message: None,
                message_hash: None,
                changed: false,
                character: None,
                deserialized_message: None,
            },
        };
    }

    fn post_init(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn update_character(&mut self) {
        let mut char = self
            .sheet_info
            .character
            .clone()
            .unwrap_or(Character::new_empty());

        char.spell_block = Some(
            self.sheet_info
                .jsonified_message
                .clone()
                .expect("Character sheet should always generate jsonified message"),
        );
        char.spell_block_hash = self.sheet_info.message_hash.clone();

        self.sheet_info.character = Some(char);
    }

    fn mut_sheet_info(&mut self) -> &mut SheetInfo {
        &mut self.sheet_info
    }
    fn sheet_info(&self) -> &SheetInfo {
        &self.sheet_info
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

        Err(Box::new(stat_puller::StatPullerError::NoSpellSheet))
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
