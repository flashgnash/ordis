use crate::common::fetch_message_poise;
use crate::common::Context;
use crate::common::Error;

use sha2::{Digest, Sha256};

use crate::db;
use crate::db::models::Character;
use crate::db::models::User;
use diesel::sqlite::SqliteConnection;

use serde_json::Value;

extern crate regex;
use std::fmt;

use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::MessageId;
use poise::CreateReply;

use super::spell_sheet::SpellSheet;
use super::stat_block::StatBlock;

use crate::gpt::generate_to_string;
use crate::gpt::Message;
use crate::gpt::Role;

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

pub struct SheetInfo {
    pub original_message: Option<String>,
    pub jsonified_message: Option<String>,
    pub message_hash: Option<String>,
    pub changed: bool,
    pub character: Option<Character>,
}
impl SheetInfo {
    fn get_json(&self) -> Result<String, Error> {
        if let Some(json) = self.jsonified_message.clone() {
            Ok(json)
        } else {
            Err(Box::new(StatPullerError::JsonNotInitialised))
        }
    }
}

pub trait CharacterSheetable: Sized + std::fmt::Display {
    const PROMPT: &'static str;

    fn new() -> Self;

    fn update_character(&mut self);

    fn mut_sheet_info(&mut self) -> &mut SheetInfo;
    fn sheet_info(&self) -> &SheetInfo;

    fn get_json(self) -> Result<String, Error> {
        let sheet_info = self.sheet_info();

        println!("sheet info {sheet_info}");

        Ok(sheet_info.get_json()?)
    }

    async fn get_sheet_message(
        ctx: &Context<'_>,
        character: &Character,
    ) -> Result<poise::serenity_prelude::Message, Error>;

    fn get_previous_hash(character: &Character) -> Option<String>;

    async fn from_string(message: &str) -> Result<Self, Error> {
        let mut instance = Self::new();

        let sheet_info = instance.mut_sheet_info();

        if let Some(original_message) = &sheet_info.original_message {
            println!("Original: {}", &original_message);
        } else {
            println!("Null!");
        }

        sheet_info.original_message = Some(message.to_string());

        if let Some(original_message) = &sheet_info.original_message {
            println!("Modified: {}", &original_message);
        } else {
            println!("Also Null!");
        }
        let model = "gpt-4o-mini";

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

        let response = generate_to_string(model, messages).await?;

        sheet_info.jsonified_message = Some(response);

        Ok(instance)
    }

    #[allow(dead_code)]
    async fn from_message(
        ctx: &Context<'_>,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<Self, Error> {
        let message = fetch_message_poise(&ctx, channel_id, message_id)
            .await?
            .content;

        return Ok(Self::from_string(&message).await?);
    }

    fn from_cache(message: &str, json: &str) -> Self {
        let mut instance = Self::new();
        let sheet_info = instance.mut_sheet_info();

        sheet_info.original_message = Some(message.to_string());
        sheet_info.jsonified_message = Some(json.to_string());

        return instance;
    }

    async fn from_character(ctx: &Context<'_>, character: &Character) -> Result<Self, Error> {
        let message = Self::get_sheet_message(ctx, &character).await?;
        let mut sheet = Self::from_message(ctx, message.channel_id, message.id).await?;

        let sheet_info = sheet.mut_sheet_info();
        sheet_info.character = Some(character.clone());

        return Ok(sheet);
    }

    async fn from_character_with_cache(
        ctx: &Context<'_>,
        character: &Character,
    ) -> Result<Self, Error> {
        let stat_message = Self::get_sheet_message(ctx, &character).await?;

        let hash_hex = crate::common::hash(&stat_message.content);

        let previous_hash = Self::get_previous_hash(character);

        let generate_new_json = match previous_hash {
            Some(value) => value != hash_hex,
            None => true,
        };

        if generate_new_json {
            println!("Generating new json via openai");
            let mut sheet = Self::from_string(&stat_message.content).await?;
            let sheet_info = sheet.mut_sheet_info();

            sheet_info.message_hash = Some(crate::common::hash(&stat_message.content));
            sheet_info.character = Some(character.clone());
            sheet_info.changed = true;
            sheet.update_character();

            Ok(sheet)
        } else {
            println!("Got cached stat block");

            let mut sheet = Self::from_cache(
                &stat_message.content,
                &character
                    .stat_block
                    .clone()
                    .expect("stat block hash has been checked"),
            );

            let sheet_info = sheet.mut_sheet_info();

            sheet_info.character = Some(character.clone());

            Ok(sheet)
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum StatPullerError {
    Generic,
    NoCharacterSheet,
    SpellNotFound,
    NoCharacterSelected,
    JsonNotInitialised,
}

impl fmt::Display for StatPullerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for StatPullerError {}

pub async fn get_user_character(
    ctx: &Context<'_>,
    db_connection: &mut SqliteConnection,
) -> Result<db::models::Character, Error> {
    let user = crate::common::get_user(ctx, db_connection).await?;

    if let Some(character_id) = user.selected_character {
        return Ok(db::characters::get(db_connection, character_id)?);
    }

    Err(Box::new(StatPullerError::NoCharacterSelected))
}

pub async fn get_sheet<T: CharacterSheetable>(ctx: &Context<'_>) -> Result<T, Error> {
    let db_connection = &mut db::establish_connection();

    let character = get_user_character(ctx, db_connection).await?;

    let character_sheet = T::from_character_with_cache(ctx, &character).await?;

    let sheet_info = character_sheet.sheet_info();

    if sheet_info.changed == true {
        let new_char = sheet_info
            .character
            .clone()
            .expect("Tried to update a non existent character?!");

        let _ = db::characters::update(db_connection, &new_char);
    }

    Ok(character_sheet)
}

#[poise::command(slash_command)]
pub async fn pull_stats(ctx: Context<'_>) -> Result<(), Error> {
    let thinking_message = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    let msg = ctx.send(thinking_message).await?;

    let stat_block: StatBlock = get_sheet(&ctx).await?;

    let reply = CreateReply::default().content(
        stat_block
            .sheet_info
            .jsonified_message
            .expect("Stat block should always generate json"),
    );
    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(
    slash_command,
    // description_localized = "Pull a single stat from your character sheet"
)]
pub async fn pull_stat(ctx: Context<'_>, stat_name: String) -> Result<(), Error> {
    let stat_block_thinking_message = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    let msg = ctx.send(stat_block_thinking_message).await?;

    // let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let stat_block: StatBlock = get_sheet(&ctx).await?;

    let stats: Value = serde_json::from_str(
        &stat_block
            .sheet_info
            .jsonified_message
            .expect("Stat block should always generate json"),
    )?;

    // println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(stats.get(stat_name).unwrap().to_string());
    msg.edit(ctx, reply).await?;

    return Ok(());
}
