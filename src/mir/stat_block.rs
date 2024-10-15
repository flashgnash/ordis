use std::fmt;

use crate::common::Context;
use crate::common::Error;

use crate::db::models::Character;

use super::stat_puller;

use poise::serenity_prelude::Message;

pub struct StatBlock {
    pub original_message: Option<String>,
    pub jsonified_message: Option<String>,
    pub message_hash: Option<String>,
    pub changed: bool,
    pub character: Option<Character>,
}

impl fmt::Display for StatBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(json) = self.jsonified_message.as_ref() {
            return write!(f, "{json}");
        }
        if let Some(message) = self.original_message.as_ref() {
            return write!(f, "{message}");
        }

        write!(f, "No stat sheet found")
    }
}
impl super::stat_puller::CharacterSheetable for StatBlock {
    fn new() -> Self {
        return Self {
            original_message: None,
            jsonified_message: None,
            message_hash: None,
            changed: false,
            character: None,
        };
    }

    //Boilerplate (field mappings for the trait to use)
    fn get_previous_hash(character: &Character) -> Option<String> {
        return character.stat_block_hash.clone();
    }
    fn get_hash(&self) -> Option<String> {
        self.message_hash.clone()
    }
    fn set_hash(&mut self, hash: String) {
        self.message_hash = Some(hash);
    }
    fn set_character(&mut self, char: Character) {
        self.character = Some(char.clone());
    }
    fn get_character(&self) -> Option<Character> {
        return self.character.clone();
    }
    fn get_changed(&self) -> bool {
        self.changed
    }
    fn set_changed(&mut self, value: bool) {
        self.changed = value;
    }
    fn jsonified_message_mut(&mut self) -> &mut Option<String> {
        &mut self.jsonified_message
    }
    fn original_message_mut(&mut self) -> &mut Option<String> {
        &mut self.original_message
    }

    fn update_character(&mut self) {
        let mut char = self.character.clone().unwrap_or(Character::new_empty());

        char.stat_block = Some(
            self.jsonified_message_mut()
                .clone()
                .expect("Character sheet should always generate jsonified message"),
        );
        char.stat_block_hash = self.message_hash.clone();

        self.character = Some(char);
    }

    async fn get_sheet_message(ctx: &Context<'_>, character: &Character) -> Result<Message, Error> {
        if let (Some(channel_id_u64), Some(message_id_u64)) = (
            character.stat_block_channel_id.clone(),
            character.stat_block_message_id.clone(),
        ) {
            let channel_id = channel_id_u64.parse().expect("Invalid channel ID");
            let message_id = message_id_u64.parse().expect("Invalid message ID");

            let message = crate::common::fetch_message_poise(&ctx, channel_id, message_id).await?;

            return Ok(message);
        }

        Err(Box::new(stat_puller::StatPullerError::NoCharacterSheet))
    }

    const PROMPT: &'static str = r#"
        You are a stat pulling program. 
        Following this prompt you will receive a block of stats.
        Use the following schema:
        {
            "name": (string),
            "level": (number),
            "hunger": (number),
   
            "actions": (number),
            "reactions": (number),
    
            "speed": (number),
            "armor": (number),
            "hp": (number),
            "current_hp": (number),
            "hpr": (number),

            "energy_pool": (number),            
    
            "hit_die_per_level": (number)d(number),
            "stat_die_per_level": (number)d(number),
            "spell_die_per_level": (number)d(number),
            "stat_points_saved": (number)d(number),
            "spell_points_saved": (number)d(number),

    
            "stats": {
                "str": (number),
                "agl": (number),
                "con": (number),
                "wis": (number),
                "int": (number),
                "cha": (number),
                "kno": (number),
            }
        }    
        If there are missing values, interpret them as null
        If you are expecting a value in a specific format but it is incorrect, instead set the value as 'ERROR - (explanation)'
        You should translate these stats into a json dictionary.
        All keys should be lower case and spell corrected. Respond with only valid json
    "#;
}
