use crate::common::fetch_message_poise;
use crate::common::Context;
use crate::common::Error;

use crate::gpt::generate_to_string;
use crate::gpt::Message;
use crate::gpt::Role;

mod db;
use crate::db::models::User;
use serde_json::Value;

extern crate regex;
use std::fmt;

use poise::serenity_prelude::ChannelId;
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

    println!("{}", &preprompt);
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

async fn get_stat_block_json(ctx: Context<'_>) -> Result<String, Error> {
    let author = &ctx.author();
    let user_id = author.id.get();
    let db_connection = &mut db::establish_connection();
    let user_result = db::users::get(db_connection, user_id);

    let stat_block_hash = user_result.stat_block_hash;

    let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let mut generate_new_json: bool;

    match stat_block_hash {
        Some(value) => {}
        None => {
            generate_new_json = true;
        }
    }
    let mut response_message: String;

    if generate_new_json {
        response_message = generate_statpuller(&stat_message.content).await?;
    }

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

    println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(response_message);
    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(slash_command, prefix_command)]
pub async fn pull_stat(
    ctx: Context<'_>,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
    stat_name: String,
) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let response_message = generate_statpuller(&stat_message.content).await?;

    let stats: Value = serde_json::from_str(&response_message)?;

    // println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(stats.get(stat_name).unwrap().to_string());
    msg.edit(ctx, reply).await?;

    return Ok(());
}
