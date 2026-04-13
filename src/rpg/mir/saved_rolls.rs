// use std::fmt;

// use crate::common::Error;

// use crate::db::models::Character;

// use super::super::CharacterSheetable;
// use super::super::RpgError;
// use super::super::SheetInfo;

// use poise::serenity_prelude::Message;

// pub struct SavedRollSheet {
//     pub sheet_info: SheetInfo,
//     pub character_id: Option<i32>,
//     pub saved_rolls: Option<Vec<SavedRoll>>,
// }

// #[derive(Clone)]
// pub struct SavedRoll {
//     pub name: String,
//     pub formula: String,
// }

// impl fmt::Display for SavedRollSheet {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // if let (Some(name), Some(formula)) = (&self.name, &self.formula) {}

//         write!(f, "I can't be bothered to do this right now")
//     }
// }

// impl CharacterSheetable for SavedRollSheet {
//     fn new() -> Self {
//         return Self {
//             character_id: None,

//             saved_rolls: None,

//             sheet_info: SheetInfo {
//                 original_message: None,
//                 jsonified_message: None,
//                 message_hash: None,
//                 changed: false,
//                 character: None,
//                 deserialized_message: None,
//             },
//         };
//     }

//     fn post_init(&mut self) -> Result<(), Error> {
//         let deserialized_message = self
//             .sheet_info
//             .deserialized_message
//             .as_ref()
//             .expect("This should be set before calling post_init");

//         Ok(())
//     }

//     fn update_character(&mut self) {
//         let mut char = self
//             .sheet_info
//             .character
//             .clone()
//             .unwrap_or(Character::new_empty());

//         char.stat_block = Some(
//             self.sheet_info
//                 .jsonified_message
//                 .clone()
//                 .expect("Character sheet should always generate jsonified message"),
//         );

//         char.stat_block_hash = self.sheet_info.message_hash.clone();

//         self.sheet_info.character = Some(char);
//     }

//     fn mut_sheet_info(&mut self) -> &mut SheetInfo {
//         &mut self.sheet_info
//     }
//     fn sheet_info(&self) -> &SheetInfo {
//         &self.sheet_info
//     }

//     fn get_previous_block(character: &Character) -> (Option<String>, Option<String>) {
//         return (
//             character.stat_block_hash.clone(),
//             character.stat_block.clone(),
//         );
//     }

//     async fn get_sheet_message(
//         ctx: &poise::serenity_prelude::Context,
//         character: &Character,
//     ) -> Result<Message, Error> {
//         if let (Some(channel_id_u64), Some(message_id_u64)) = (
//             character.stat_block_channel_id.clone(),
//             character.stat_block_message_id.clone(),
//         ) {
//             let channel_id = channel_id_u64.parse().expect("Invalid channel ID");
//             let message_id = message_id_u64.parse().expect("Invalid message ID");

//             let message = crate::common::fetch_message(&ctx, channel_id, message_id).await?;

//             return Ok(message);
//         }

//         Err(Box::new(RpgError::NoCharacterSheet))
//     }

//     const PROMPT: &'static str = r#"
//         You are a saved roll reading program.
//         Following this prompt you will receive a key value pair list of roll formulas and their names.
//         Use the following schema:
//         {

//             "my_custom_roll": (string),
//             "my_other_roll": (string)

//         }

//         All keys should be lower case and spell corrected. Respond with only valid, minified json

//         DO NOT USE BACKTICKS OR BACKSLASHES IN YOUR RESPONSE

//     "#;
// }
