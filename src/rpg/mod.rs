pub mod mir;
pub mod spells;

use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tokio::time::sleep;

use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;


// use crate::db;
use crate::db::models::Character;
// use crate::db::models::User;
// use diesel::sqlite::SqliteConnection;

use serde_json::Value;

use crate::common::fetch_message;
use crate::common::Context;
use crate::common::Error;
use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::MessageId;

use crate::llm::generate_to_string;
use crate::llm::Message;
use crate::llm::Role;

use crate::db;

use diesel::sqlite::SqliteConnection;

#[derive(Clone)]
pub struct SheetInfo {
    pub original_message: Option<String>,
    pub jsonified_message: Option<String>,
    pub deserialized_message: Option<Value>,

    pub message_hash: Option<String>,
    pub changed: bool,
    pub character: Option<Character>,
}


#[allow(dead_code)]
#[derive(Debug)]
pub enum RpgError {
    NoGuildId,

    NoCharacterSheet,
    NoCharacterSelected,

    NoSpellSheet,
    SpellNotFound,
    NoSpellCost,
    NoMaxEnergy,
    GaugeMessageMissing,

    NoEnergyDie,
    NoMagicDie,
    NoTrainingDie,

    JsonNotInitialised,
    TestingError,

    InvalidCharacterError
}

impl fmt::Display for RpgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpgError::NoGuildId => write!(f, "Guild ID is missing - Are you in a DM?"),
            RpgError::NoCharacterSheet => {
                write!(f, "Character sheet is missing - Was it deleted?")
            }
            RpgError::NoCharacterSelected => write!(f, 
                "No character selected (please select one with /characters and /select_character (id))"
            ),
            RpgError::NoSpellSheet => write!(f, "Spell sheet is missing - please set one with the Set Spell Message button"),
            RpgError::SpellNotFound => write!(f, "Spell not found"),
            RpgError::NoSpellCost => write!(f, "Spell cost appears to be missing from your spell block"),
            RpgError::NoMaxEnergy => write!(f, "Energy pool appears to be missing from your stat block"),
            RpgError::GaugeMessageMissing => write!(f, "Gauge message is missing - was it deleted?"),
            RpgError::NoEnergyDie => write!(f, "Energy die per level appears to be missing from your stat block"),
            RpgError::NoMagicDie => write!(f, "Magic die per level appears to be missing from your stat block"),
            RpgError::NoTrainingDie => write!(f, "Training die per level appears to be missing from your stat block"),
            RpgError::JsonNotInitialised => write!(f, "JSON is not initialised - this should never happen"),
            _ => write!(f,"Testing"),
        }
    }
}
impl std::error::Error for RpgError {}




impl fmt::Display for SheetInfo {
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





pub trait CharacterSheetable: Sized + std::fmt::Display + Send + Sync + Clone {
    const PROMPT: &'static str;

    fn new() -> Self;

    fn post_init(&mut self) -> Result<(), Error>;

    fn update_character(&mut self);

    fn mut_sheet_info(&mut self) -> &mut SheetInfo;
    fn sheet_info(&self) -> &SheetInfo;

    async fn get_sheet_message(
        ctx: &poise::serenity_prelude::Context, // 'a is to specify message is borrowed from context
        character: &Character,
    ) -> Result<poise::serenity_prelude::Message, Error>;

    fn get_previous_block(character: &Character) -> (Option<String>, Option<String>); //Hash, message

    async fn from_string(message: &str) -> Result<Self, Error> {
        let mut instance = Self::new();

        let sheet_info = instance.mut_sheet_info();

        sheet_info.original_message = Some(message.to_string());

        let mut response_filtered: String = "asdhjka".to_string();

        let mut valid = false;

        println!("Attempting to get json from openai");


        while ! valid {
            println!("Attempt");
            let preprompt = Self::PROMPT.to_string();

            let messages = vec![
                Message {
                    role: Role::system,
                    content: preprompt,
                },
                Message {
                    role: Role::user,
                    content: message.to_string(),
                },
            ];

            let response = generate_to_string(None, messages).await?;
            let cleaned = response.replace("```json", "").replace("```", "");

            if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
              valid = true;
              response_filtered = cleaned;
            }
            else{
                println!("Failed");
                sleep(Duration::from_millis(500)).await;
            }

        };

       
        sheet_info.jsonified_message = Some(response_filtered);

        sheet_info.deserialized_message = Some(serde_json::from_str(
            &sheet_info
                .jsonified_message
                .clone()
                .expect("This property was just set"),
        )?);

        instance.post_init()?;

        Ok(instance)
    }

    #[allow(dead_code)]
    async fn from_message(
        ctx: &poise::serenity_prelude::Context,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<Self, Error> {
        let message = fetch_message(&ctx, channel_id, message_id)
            .await?
            .content;

        return Ok(Self::from_string(&message).await?);
    }

    fn from_json(message: Option<&str>, json: &str) -> Result<Self, Error> {
        let mut instance = Self::new();
        let sheet_info = instance.mut_sheet_info();

        sheet_info.original_message = message.and_then(|m| Some(m.to_string()));
        sheet_info.jsonified_message = Some(json.to_string());

        sheet_info.deserialized_message = Some(serde_json::from_str(
            &sheet_info
                .jsonified_message
                .clone()
                .expect("This property was literally just set"),
        )?);

        instance.post_init()?;

        Ok(instance)
    }


    async fn message_changed(
        ctx: &poise::serenity_prelude::Context,
        character: &Character,
    ) -> Result<bool, Error> {

        let stat_message = Self::get_sheet_message(ctx, &character).await?;

        let hash_hex = crate::common::hash(&stat_message.content);

        let (previous_hash, _) = Self::get_previous_block(character);

        // if let Some(prev) = &previous_hash {
            
        //     println!("Prev: {prev}\n Current:{hash_hex}");
        // }
        // else{
        //    println!("No prev hash") 
        // }

        
        Ok(match previous_hash {
            Some(value) => value != hash_hex,
            None => true,
        })

    }
    //If the character sheet has changed, generate a new one with openAI
    async fn from_character_openai(
        ctx: &poise::serenity_prelude::Context,
        character: &Character,
    ) -> Result<Self, Error> {

        let stat_message = Self::get_sheet_message(ctx, &character).await?;

        // let generate_new_json = Self::message_changed(ctx,character).await?;
        
        println!("Generating new json via openai");
        let mut sheet = Self::from_string(&stat_message.content).await?;
        let sheet_info = sheet.mut_sheet_info();

        println!("b");

        sheet_info.message_hash = Some(crate::common::hash(&stat_message.content));
        sheet_info.character = Some(character.clone());
        sheet_info.changed = true;

        println!("c");
        sheet.update_character();

        println!("d");
        Ok(sheet)

    }
    async fn from_character_database(
        ctx: Option<&poise::serenity_prelude::Context>,
        character: &Character,
    ) -> Result<Self, Error> {


        let (_previous_hash, previous_block) = Self::get_previous_block(character);

        let mut sheet: Self;

        if let Some(ctx) = ctx {
            
            let stat_message = Self::get_sheet_message(ctx, &character).await?;

            if let Some(prev_block) = previous_block {

                    
                sheet = Self::from_json(
                    Some(&stat_message.content),
                    &prev_block
                )?;
            }

            else {
                sheet = Self::from_string(&stat_message.content).await?
            }
        }
        else {
            sheet = Self::from_json(None,&character.stat_block.clone().ok_or(RpgError::NoCharacterSheet)?)?;
        }

        let sheet_info = sheet.mut_sheet_info();

        sheet_info.character = Some(character.clone());

        println!("Getting saved stat block");
        Ok(sheet)
    }


        

}
pub async fn get_user_character(
    ctx: &Context<'_>
) -> Result<Option<db::models::Character>, Error> {
    let user = crate::common::get_user(ctx ).await?;

    if let Some(character_id) = user.selected_character {
        return Ok(Some(db::characters::get(character_id)?));
    }

    Ok(None)
}

lazy_static! {
    static ref SHEET_CACHE: Mutex<HashMap<(TypeId,i32), Box<dyn Any + Send + Sync>>> =
        Mutex::new(HashMap::new());
}



pub async fn get_sheet_of_sender<T: CharacterSheetable + 'static>(ctx: &Context<'_>) -> Result<Option<T>,Error> {

    if let Some(character) =  &get_user_character(ctx).await? {
        
        let sheet = get_sheet(
          ctx.serenity_context(),
          character
        ).await?; 

        return Ok(Some(sheet))
    }
    Ok(None)
}
pub async fn get_sheet<T: CharacterSheetable + 'static>(ctx: &poise::serenity_prelude::Context,character: &Character) -> Result<T, Error> {

    let mut cache = SHEET_CACHE.lock().await;


    let key = (TypeId::of::<T>(),character.id.expect("Char ID should not be null"));

    

    if cache.contains_key(&key) {
        if T::message_changed(ctx, &character).await? {
            println!("Fetching from cache");

            let sheet = T::from_character_openai(ctx, &character).await?;
            cache.insert(key,Box::new(sheet));       
        }
    }
    else {

        println!("Not cached - generating");
        
       let sheet = T::from_character_database(Some(ctx), &character).await?;
        cache.insert(key,Box::new(sheet));       
    }

    let character_sheet = cache.get(&key).and_then(|a| a.downcast_ref::<T>()).ok_or(RpgError::NoCharacterSheet)?; 

    let sheet_info = character_sheet.sheet_info();

    if sheet_info.changed == true {
        let new_char = sheet_info
            .character
            .clone()
            .expect("Tried to update a non existent character?!");

        let _ = db::characters::update(&new_char);
    }

    Ok(character_sheet.clone())
}
