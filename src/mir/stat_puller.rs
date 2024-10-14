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

pub trait FromDiscordMessage: Sized {
    const PROMPT: &'static str;

    fn new() -> Self;

    fn jsonified_message_mut(&mut self) -> &mut Option<String>;
    fn original_message_mut(&mut self) -> &mut Option<String>;

    // fn get_message_reference_from_db(connection: SqliteConnection) -> u64;

    async fn from_string(message: &str) -> Result<Self, Error> {
        let mut instance = Self::new();

        *instance.original_message_mut() = Some(message.to_string());

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

        *instance.jsonified_message_mut() = Some(response);

        Ok(instance)
    }

    #[allow(dead_code)]
    async fn from_message(
        ctx: Context<'_>,
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

        *instance.original_message_mut() = Some(message.to_string());
        *instance.jsonified_message_mut() = Some(json.to_string());

        return instance;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum StatPullerError {
    Generic,
    NoCharacterSheet,
    SpellNotFound,
    NoCharacterSelected,
}

impl fmt::Display for StatPullerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for StatPullerError {}

pub async fn get_user(
    ctx: &Context<'_>,
    db_connection: &mut SqliteConnection,
) -> Result<User, Error> {
    let author = &ctx.author();

    let user_id = author.id.get();

    Ok(db::users::get_or_create(db_connection, user_id)?)
}

pub async fn get_user_character(
    ctx: &Context<'_>,
    db_connection: &mut SqliteConnection,
) -> Result<db::models::Character, Error> {
    let user = get_user(ctx, db_connection).await?;

    if let Some(character_id) = user.selected_character {
        return Ok(db::characters::get(db_connection, character_id)?);
    }

    Err(Box::new(StatPullerError::NoCharacterSelected))
}

pub async fn get_spell_block_json(ctx: &Context<'_>) -> Result<SpellSheet, Error> {
    let db_connection = &mut db::establish_connection();

    let mut character = get_user_character(ctx, db_connection).await?;

    if let (Some(channel_id_u64), Some(message_id_u64)) = (
        character.spell_block_channel_id.clone(),
        character.spell_block_message_id.clone(),
    ) {
        let channel_id = ChannelId::new(channel_id_u64.parse().expect("Invalid ChannelID"));
        let message_id = MessageId::new(message_id_u64.parse().expect("Invalid MessageID"));

        let spell_block_hash = &character.spell_block_hash;

        let spell_message = fetch_message_poise(&ctx, channel_id, message_id).await?;
        let hash_hex = crate::common::hash(&spell_message.content);

        let generate_new_json = match spell_block_hash {
            Some(value) => value != &hash_hex,
            None => true,
        };

        let spell_sheet: SpellSheet;

        if generate_new_json {
            spell_sheet = SpellSheet::from_string(&spell_message.content).await?;

            let json = spell_sheet
                .jsonified_message
                .clone()
                .expect("Spell sheet should always contain json");

            println!("Generated new json");
            character.spell_block = Some(json);
            character.spell_block_hash = Some(hash_hex);

            let _ = db::characters::update(db_connection, &character);
        } else {
            println!("Got cached spell block");
            spell_sheet = SpellSheet::from_cache(
                &spell_message.content,
                &character
                    .spell_block
                    .expect("Spell block hash has been checked"),
            )
        }

        return Ok(spell_sheet);
    }
    return Err(Box::new(StatPullerError::NoCharacterSheet));
}

pub async fn get_stat_block_json(ctx: &Context<'_>) -> Result<StatBlock, Error> {
    let db_connection = &mut db::establish_connection();

    let mut character = get_user_character(ctx, db_connection).await?;

    if let (Some(channel_id_u64), Some(message_id_u64)) = (
        character.stat_block_channel_id.clone(),
        character.stat_block_message_id.clone(),
    ) {
        let channel_id = ChannelId::new(channel_id_u64.parse().expect("Invalid ChannelID"));
        let message_id = MessageId::new(message_id_u64.parse().expect("Invalid MessageID"));

        let stat_block_hash = &character.stat_block_hash;

        let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;
        let hash_hex = crate::common::hash(&stat_message.content);

        let generate_new_json = match stat_block_hash {
            Some(value) => value != &hash_hex,
            None => true,
        };

        let stat_sheet: StatBlock;

        if generate_new_json {
            stat_sheet = StatBlock::from_string(&stat_message.content).await?;

            let json = stat_sheet
                .jsonified_message
                .clone()
                .expect("stat sheet should always contain json");

            println!("Generated new json");
            character.stat_block = Some(json);
            character.stat_block_hash = Some(hash_hex);

            let _ = db::characters::update(db_connection, &character);
        } else {
            println!("Got cached stat block");
            stat_sheet = StatBlock::from_cache(
                &stat_message.content,
                &character
                    .stat_block
                    .expect("stat block hash has been checked"),
            )
        }

        return Ok(stat_sheet);
    }
    return Err(Box::new(StatPullerError::NoCharacterSheet));
    // let message_id_u64: u64 = user
    //     .stat_block_message_id
    //     .clone()
    //     .expect("No message ID saved")
    //     .parse()
    //     .expect("Invalid MessageId");
}

#[poise::command(slash_command)]
pub async fn pull_stats(ctx: Context<'_>) -> Result<(), Error> {
    let thinking_message = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    let msg = ctx.send(thinking_message).await?;

    let stat_block = get_stat_block_json(&ctx).await?;

    let reply = CreateReply::default().content(
        stat_block
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

    let msg = ctx.say("*Thinking, please wait...*").await?;

    // let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let stat_block = get_stat_block_json(&ctx).await?;

    let stats: Value = serde_json::from_str(
        &stat_block
            .jsonified_message
            .expect("Stat block should always generate json"),
    )?;

    // println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(stats.get(stat_name).unwrap().to_string());
    msg.edit(ctx, reply).await?;

    return Ok(());
}
