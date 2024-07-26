use crate::common::Context;
use crate::common::Error;

use crate::gpt::generate_to_string;
use crate::gpt::Message;
use crate::gpt::Role;

extern crate regex;
use std::fmt;

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
                You should translate these stats into a json dictionary.
                All keys should be lower case and spell corrected. Respond with only valid json"
    )
    .to_string();

    println!("{}", &preprompt);
    let messages = vec![
        Message {
            role: Role::System,
            content: preprompt,
        },
        Message {
            role: Role::User,
            content: message.to_string(),
        },
    ];

    let response = generate_to_string(model, messages).await?;

    return Ok(response);
}

#[poise::command(slash_command, prefix_command)]
pub async fn pull_stats(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response_message = generate_statpuller(&message).await?;

    println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}
