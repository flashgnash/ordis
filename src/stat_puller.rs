use crate::common::fetch_message_poise;
use crate::common::Context;
use crate::common::Error;

use crate::gpt::generate_to_string;
use crate::gpt::Message;
use crate::gpt::Role;
use sha2::{Digest, Sha256};

use crate::dice::roll_replace;

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
            "actions": (number),
            "reactions": (number),
            "speed": (number),
            "armor": (number),
            "hp": (number),
            "current_hp": (number),
            "hpr": (number),
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
                "cha": (number)
            },
            "tags": {
                "something": "1%",
                "something else": "2%",
                "another_tag": "3%"
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

async fn get_stat_block_json(ctx: &Context<'_>) -> Result<String, Error> {
    let author = &ctx.author();
    let user_id = author.id.get();
    let db_connection = &mut db::establish_connection();
    let mut user = db::users::get(db_connection, user_id)?;

    let channel_id_u64: u64 = user
        .stat_block_channel_id
        .clone()
        .expect("No channel ID saved")
        .parse()
        .expect("Invalid ChannelId");

    let message_id_u64: u64 = user
        .stat_block_message_id
        .clone()
        .expect("No message ID saved")
        .parse()
        .expect("Invalid MessageId");

    let channel_id = ChannelId::new(channel_id_u64);
    let message_id = MessageId::new(message_id_u64);

    let stat_block_hash = &user.stat_block_hash;

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
    let mut response_message: String;

    if generate_new_json {
        response_message = generate_statpuller(&stat_message.content).await?;
        println!("Generated new json");
        user.stat_block = Some(response_message.clone());

        user.stat_block_hash = Some(hash_hex);
    } else {
        response_message = user
            .stat_block
            .clone()
            .expect("Error: Stat block hash was present but stat block was not!");
        println!("Got cached stat block")
    }

    let _ = db::users::update(db_connection, &user);

    Ok(response_message)
}

#[poise::command(slash_command, prefix_command)]
pub async fn pull_stats(
    ctx: Context<'_>,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let response_message = generate_statpuller(&stat_message.content).await?;

    // println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(response_message);
    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(slash_command, prefix_command)]
pub async fn pull_stat(ctx: Context<'_>, stat_name: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    // let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let response_message = get_stat_block_json(&ctx).await?;

    let stats: Value = serde_json::from_str(&response_message)?;

    // println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(stats.get(stat_name).unwrap().to_string());
    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(slash_command, prefix_command)]
pub async fn setup_character_sheet(
    ctx: Context<'_>,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<(), Error> {
    let db_connection = &mut db::establish_connection();

    let author = &ctx.author();
    let user_id = author.id.get();

    let mut user = db::users::get(db_connection, user_id)?;

    user.stat_block_message_id = Some(message_id.to_string());
    user.stat_block_channel_id = Some(channel_id.to_string());

    let _ = db::users::update(db_connection, &user);
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn level_up(ctx: Context<'_>, num_levels: i32) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response_message = get_stat_block_json(&ctx).await?;

    let stats: Value = serde_json::from_str(&response_message)?;

    let hit_die = stats.get("hit_die_per_level").unwrap().to_string();
    let stat_die = stats.get("stat_die_per_level").unwrap().to_string();
    let spell_die = stats.get("spell_die_per_level").unwrap().to_string();

    let mut message = format!(
        "Per Level: \nHit: {hit_die} \\| Stat: {stat_die} \\| Spell: {spell_die}\n------------------------------------"
    );

    for i in 1..num_levels + 1 {
        let (hit_die_result, _) = roll_replace(&hit_die.as_str())?;
        let (stat_die_result, _) = roll_replace(&stat_die.as_str())?;
        let (spell_die_result, _) = roll_replace(&spell_die.as_str())?;

        message = format!(
            "{message}\n\n{i})       :heart: {hit_die_result}         :hash: {stat_die_result}         :magic_wand: {spell_die_result}"
        );
    }
    message = message.replace('"', "");

    let reply = CreateReply::default().content(message);
    msg.edit(ctx, reply).await?;

    return Ok(());
}
