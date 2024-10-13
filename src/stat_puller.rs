use crate::common::fetch_message_poise;
use crate::common::Context;
use crate::common::Error;
use crate::gpt::generate_to_string;
use crate::gpt::Message;
use crate::gpt::Role;
use sha2::{Digest, Sha256};

use crate::db;

use serde_json::Value;

extern crate regex;
use std::fmt;

use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::MessageId;
use poise::CreateReply;

#[allow(dead_code)]
#[derive(Debug)]
pub enum StatPullerError {
    Generic,
    NoCharacterSheet,
    SpellNotFound,
}

impl fmt::Display for StatPullerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for StatPullerError {}

#[allow(dead_code)]
pub async fn generate_statpuller(message: &str) -> Result<String, Error> {
    let model = "gpt-4o-mini";

    let schema = r#"
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
    "#;

    let preprompt = format!(
        "You are a stat pulling program. 
                Following this prompt you will receive a block of stats.
                Use the following schema:
                {schema}
                If there are missing values, interpret them as null
                If you are expecting a value in a specific format but it is incorrect, instead set the value as 'ERROR - (explanation)'
                You should translate these stats into a json dictionary.
                All keys should be lower case and spell corrected. Respond with only valid json"
    )
    .to_string();

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

    return Ok(response);
}

#[allow(dead_code)]
pub async fn generate_spellpuller(message: &str) -> Result<String, Error> {
    let model = "gpt-4o-mini";

    let schema = r#"
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
    "#;

    let preprompt = format!(
        "You are a spell list pulling program. 
                Following this prompt you will receive a block of spells and their costs.
                Use the following schema:
                {schema}
                If there are missing values, interpret them as null
                For cast time, use the middle value that should look like '2 actions'
                If there are spaces in spell names, remove them, replacing them with underscores
                If you are expecting a value in a specific format but it is incorrect, instead set the value as 'ERROR - (explanation)'
                You should translate these spells into a json dictionary.
                All keys should be lower case and spell corrected. Respond with only valid json"
    )
    .to_string();

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

    return Ok(response);
}

pub async fn get_stat_block_json_from_message(
    ctx: &Context<'_>,
    channel_id: ChannelId,
    message_id: MessageId,
) -> Result<String, Error> {
    let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let response_message = generate_statpuller(&stat_message.content).await?;

    Ok(response_message)
}

pub async fn get_spell_block_json_from_message(
    ctx: &Context<'_>,
    channel_id: ChannelId,
    message_id: MessageId,
) -> Result<String, Error> {
    let spell_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let response_message = generate_spellpuller(&spell_message.content).await?;

    Ok(response_message)
}

pub async fn get_spell_block_json(ctx: &Context<'_>) -> Result<(String, String), Error> {
    let author = &ctx.author();
    let user_id = author.id.get();
    let db_connection = &mut db::establish_connection();
    let user = db::users::get_or_create(db_connection, user_id)?;

    if let Some(character_id) = user.selected_character {
        let mut character = db::characters::get(db_connection, character_id)?;
        // let channel_id_u64: u64 = user
        //     .spell_block_channel_id
        //     .clone()
        //     .expect("No channel ID saved")

        if let Some(channel_id_u64) = character.spell_block_channel_id.clone() {
            let channel_id_parsed = channel_id_u64.parse().expect("Invalid ChannelID");
            let channel_id = ChannelId::new(channel_id_parsed);

            if let Some(message_id_u64) = character.spell_block_message_id.clone() {
                let message_id_parsed = message_id_u64.parse().expect("Invalid MessageID");
                let message_id = MessageId::new(message_id_parsed);

                let spell_block_hash = &character.spell_block_hash;

                let spell_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

                let mut generate_new_json: bool = false;

                let mut hasher = Sha256::new();
                hasher.update(&spell_message.content);
                let result = hasher.finalize();
                let hash_hex = format!("{:x}", result);

                match spell_block_hash {
                    Some(value) => {
                        if value != &hash_hex {
                            generate_new_json = true;
                        }
                    }
                    None => {
                        generate_new_json = true;
                    }
                }

                let response_message: String;

                if generate_new_json {
                    response_message = generate_spellpuller(&spell_message.content).await?;
                    println!("Generated new json");
                    character.spell_block = Some(response_message.clone());

                    character.spell_block_hash = Some(hash_hex);
                } else {
                    response_message = character
                        .spell_block
                        .clone()
                        .expect("Error: spell block hash was present but spell block was not!");
                    println!("Got cached spell block")
                }

                let _ = db::characters::update(db_connection, &character);

                return Ok((response_message, spell_message.content));
            }
        }
    }
    return Err(Box::new(StatPullerError::NoCharacterSheet));
    // let message_id_u64: u64 = user
    //     .stat_block_message_id
    //     .clone()
    //     .expect("No message ID saved")
    //     .parse()
    //     .expect("Invalid MessageId");
}

pub async fn get_stat_block_json(ctx: &Context<'_>) -> Result<(String, String), Error> {
    let author = &ctx.author();
    let user_id = author.id.get();
    let db_connection = &mut db::establish_connection();
    let user = db::users::get_or_create(db_connection, user_id)?;

    if let Some(character_id) = user.selected_character {
        let mut character = db::characters::get(db_connection, character_id)?;
        // let channel_id_u64: u64 = user
        //     .stat_block_channel_id
        //     .clone()
        //     .expect("No channel ID saved")

        if let Some(channel_id_u64) = character.stat_block_channel_id.clone() {
            let channel_id_parsed = channel_id_u64.parse().expect("Invalid ChannelID");
            let channel_id = ChannelId::new(channel_id_parsed);

            if let Some(message_id_u64) = character.stat_block_message_id.clone() {
                let message_id_parsed = message_id_u64.parse().expect("Invalid MessageID");
                let message_id = MessageId::new(message_id_parsed);

                let stat_block_hash = &character.stat_block_hash;

                let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

                let mut generate_new_json: bool = false;

                let mut hasher = Sha256::new();
                hasher.update(&stat_message.content);
                let result = hasher.finalize();
                let hash_hex = format!("{:x}", result);

                match stat_block_hash {
                    Some(value) => {
                        if value != &hash_hex {
                            generate_new_json = true;
                        }
                    }
                    None => {
                        generate_new_json = true;
                    }
                }

                let response_message: String;

                if generate_new_json {
                    response_message = generate_statpuller(&stat_message.content).await?;
                    println!("Generated new json");
                    character.stat_block = Some(response_message.clone());

                    character.stat_block_hash = Some(hash_hex);
                } else {
                    response_message = character
                        .stat_block
                        .clone()
                        .expect("Error: Stat block hash was present but stat block was not!");
                    println!("Got cached stat block")
                }

                let _ = db::characters::update(db_connection, &character);

                return Ok((response_message, stat_message.content));
            }
        }
    }
    return Err(Box::new(StatPullerError::NoCharacterSheet));
    // let message_id_u64: u64 = user
    //     .stat_block_message_id
    //     .clone()
    //     .expect("No message ID saved")
    //     .parse()
    //     .expect("Invalid MessageId");
}

#[poise::command(
    slash_command,
    // description_localized = "Pull all stats from your character sheet (only you will be able to see the result of this command)"
)]
pub async fn pull_stats(ctx: Context<'_>) -> Result<(), Error> {
    let thinking_message = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    let msg = ctx.send(thinking_message).await?;

    let (response_message, _) = get_stat_block_json(&ctx).await?;

    let reply = CreateReply::default().content(response_message);
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

    let (response_message, _) = get_stat_block_json(&ctx).await?;

    let stats: Value = serde_json::from_str(&response_message)?;

    // println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(stats.get(stat_name).unwrap().to_string());
    msg.edit(ctx, reply).await?;

    return Ok(());
}
