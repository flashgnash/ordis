use std::fmt;

use crate::common::Error;

use crate::db::models::Character;

use super::super::CharacterSheetable;
use super::super::RpgError;
use super::super::SheetInfo;

use poise::serenity_prelude::Message;

#[derive(Clone)]
pub struct StatBlock {
    pub sheet_info: SheetInfo,
    pub stats: Option<serde_json::Value>,
    pub energy_pool: Option<i64>,
    pub hp: Option<i64>,
    pub max_hp: Option<i64>,

    pub armour: Option<i64>,
    pub max_armour: Option<i64>,

    pub soul: Option<i64>,
    pub max_soul: Option<i64>,

    pub hunger: Option<i64>,

    pub default_roll: Option<String>,
    pub modifier_formula: Option<String>,
}

impl fmt::Display for StatBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(json) = self.sheet_info.jsonified_message.as_ref() {
            return write!(f, "{json}");
        }
        if let Some(message) = self.sheet_info.original_message.as_ref() {
            return write!(f, "{message}");
        }

        write!(f, "No stat sheet found")
    }
}

impl CharacterSheetable for StatBlock {
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

            stats: None,
            energy_pool: None,

            max_hp: None,
            hp: None,

            max_armour: None,
            armour: None,

            max_soul: None,
            soul: None,

            hunger: None,

            default_roll: None,
            modifier_formula: None,
        };
    }

    fn post_init(&mut self) -> Result<(), Error> {
        let deserialized_message = self
            .sheet_info
            .deserialized_message
            .as_ref()
            .expect("This should be set before calling post_init");

        if let Some(stats) = deserialized_message.get("stats") {
            self.stats = Some(stats.clone());
        }

        self.energy_pool = deserialized_message
            .get("energy_pool")
            .and_then(|v| v.as_i64());

        self.hp = deserialized_message
            .get("current_hp")
            .and_then(|v| v.as_i64());

        self.max_hp = deserialized_message.get("hp").and_then(|v| v.as_i64());

        self.soul = deserialized_message
            .get("current_soul")
            .and_then(|v| v.as_i64());

        self.max_soul = deserialized_message.get("soul").and_then(|v| v.as_i64());

        self.armour = deserialized_message
            .get("current_armour")
            .and_then(|v| v.as_i64());

        self.max_armour = deserialized_message.get("armour").and_then(|v| v.as_i64());

        self.hunger = deserialized_message.get("hunger").and_then(|v| v.as_i64());

        self.default_roll = deserialized_message
            .get("default_roll")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or(Some("1d100".to_string()));

        self.modifier_formula = deserialized_message
            .get("modifier_formula")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(())
    }

    fn update_character(&mut self) {
        let mut char = self
            .sheet_info
            .character
            .clone()
            .unwrap_or(Character::new_empty());

        char.stat_block = Some(
            self.sheet_info
                .jsonified_message
                .clone()
                .expect("Character sheet should always generate jsonified message"),
        );

        char.stat_block_hash = self.sheet_info.message_hash.clone();

        self.sheet_info.character = Some(char);
    }

    fn mut_sheet_info(&mut self) -> &mut SheetInfo {
        &mut self.sheet_info
    }
    fn sheet_info(&self) -> &SheetInfo {
        &self.sheet_info
    }

    fn get_previous_block(character: &Character) -> (Option<String>, Option<String>) {
        return (
            character.stat_block_hash.clone(),
            character.stat_block.clone(),
        );
    }

    async fn get_sheet_message(
        ctx: &poise::serenity_prelude::Context,
        character: &Character,
    ) -> Result<Message, Error> {
        if let (Some(channel_id_u64), Some(message_id_u64)) = (
            character.stat_block_channel_id.clone(),
            character.stat_block_message_id.clone(),
        ) {
            let channel_id = channel_id_u64.parse().expect("Invalid channel ID");
            let message_id = message_id_u64.parse().expect("Invalid message ID");

            let message = crate::common::fetch_message(&ctx, channel_id, message_id).await?;

            return Ok(message);
        }

        Err(Box::new(RpgError::NoCharacterSheet))
    }

    const PROMPT: &'static str = r#"
        You are a stat pulling program. 
        Following this prompt you will receive a block of stats.
        Use the following schema:
        {
            "name": (string),
            "level": (number),
            "hunger": (number),

            "default_roll": (string),
           
            "modifier_formula": (string),
   
            "actions": (number),
            "reactions": (number),
    
            "speed": (number),
            
            "armour": (number),
            "current_armour": (number),

            "soul": (number),
            "current_soul": (number),

            "hp": (number),
            "current_hp": (number),
            "hpr": (number),

            "energy_pool": (number),            
    
            "energy_die_per_level": (number)d(number),
            "magic_die_per_level": (number)d(number),
            "training_die_per_level": (number)d(number),

    
            "stats": {
                "str": (number),
                "agl": (number),
                "con": (number),
                "wis": (number),
                "int": (number),
                "cha": (number),
                "kno": (number),

                "body": (number),
                "mobility": (number),
                "arcane": (number),
                "intuition": (number),
            }
        }    
        If there are missing values, ignore them (don't make a key for them)
        Particularly with stats it is IMPERATIVE that keys are not included if not present in the sheet
        You should translate these stats into a minified json dictionary.
        All keys should be lower case and spell corrected. Respond with only valid, minified json

        DO NOT USE BACKTICKS OR BACKSLASHES IN YOUR RESPONSE

    "#;
}
