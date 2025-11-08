use crate::common::Context;
use crate::common::Error;
use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::Colour;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateMessage;
use poise::CreateReply;
use rand::prelude::*;
use serde::Serialize;

use crate::common::safe_to_number;

use crate::common::join_to_string;
use crate::common::sum_array;

use meval::eval_str;

extern crate regex;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize)]
pub struct RollResult {
    pub message: String,
    pub result: f64,
    pub rolls: Vec<Roll>,
}

#[derive(Serialize)]
pub struct Roll {
    pub result: i32,
    pub expression: String,
}

#[derive(Debug)]
pub enum DiceError {
    TooMuchD,
    InvalidFaceCount,
    InvalidDiceCount,
}

impl fmt::Display for DiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DiceError {}

pub fn roll_dice_string(instance: &str) -> Result<Vec<Roll>, DiceError> {
    let mut number_of_dice = 1;
    let faces_of_die;

    let components: Vec<&str> = instance.split('d').collect();
    if components[0] == "" {
        faces_of_die = safe_to_number(components[1]);
    } else if components.len() == 2 {
        faces_of_die = safe_to_number(components[1]);
        number_of_dice = safe_to_number(components[0]);
    } else {
        return Err(DiceError::TooMuchD);
    }

    if number_of_dice == 0 {
        return Err(DiceError::InvalidDiceCount);
    }
    if faces_of_die == 0 {
        return Err(DiceError::InvalidFaceCount);
    }
    if faces_of_die == 1 {
        return Err(DiceError::InvalidFaceCount);
    }

    let dice_rolls = generate_randoms(number_of_dice, faces_of_die);

    Ok(dice_rolls)
}

pub fn join_rolls_to_string(rolls: &[Roll], separator: &str) -> String {
    let s: String = rolls
        .iter()
        .map(|i| i.result.to_string())
        .collect::<Vec<String>>()
        .join(separator);
    return s;
}

pub fn sum_roll_array(arr: &Vec<Roll>) -> i32 {
    let mut result = 0;

    for num in arr {
        result = result + num.result;
    }
    return result;
}
pub fn eval_roll(input: &str) -> Result<RollResult, Error> {
    let pattern = Regex::new(r"\d+d\d+").unwrap(); // This regex pattern matches three-letter words

    let mut replaced = input.to_string();

    let mut all_rolls: Vec<Roll> = vec![];

    for mat in pattern.find_iter(input) {
        // Has to be mutable to go into mutable array
        // Probably a better way but I have no net connection to check
        let mut rolls = roll_dice_string(mat.as_str())?;

        let mat_str = mat.as_str();

        replaced = replaced.replacen(mat_str, &sum_roll_array(&rolls).to_string(), 1);

        all_rolls.append(&mut rolls);
    }

    let calc_result = eval_str(&replaced)?;

    let rolls_message = format_rolls(&all_rolls);

    let message = format!("{input} \n{rolls_message}\n\n Result: __{calc_result}__");

    Ok(RollResult {
        message: message,
        result: calc_result,
        rolls: all_rolls,
    })
}

fn generate_randoms(count: i32, faces: i32) -> Vec<Roll> {
    let mut rng = rand::thread_rng();

    let mut rolls: Vec<Roll> = vec![];

    for _i in 0..count {
        rolls.push(Roll {
            result: rng.gen_range(1..faces + 1),
            expression: format!("{count}d{faces}"),
        });
    }

    return rolls;
}

fn group_rolls(rolls: &Vec<Roll>) -> HashMap<String, Vec<&Roll>> {
    let mut map: HashMap<String, Vec<&Roll>> = HashMap::new();
    for r in rolls {
        map.entry(r.expression.clone()).or_default().push(r);
    }
    map
}

fn format_rolls(rolls: &Vec<Roll>) -> String {
    let groups = group_rolls(rolls);

    groups
        .into_iter()
        .map(|(expr, rs)| {
            let results: Vec<i32> = rs.into_iter().map(|r| r.result).collect();
            format!("- {}: {:?} ({})", expr, results, sum_array(&results))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn generate_roll_embed(
    roll_message: String,
    name: &str,
    colour: Colour,
) -> Result<CreateEmbed, Error> {
    let embed = CreateEmbed::default()
        .title(format!("Rolling for {name}..."))
        .colour(colour)
        .description(format!("\n​\n{roll_message}"));

    Ok(embed)
}

pub async fn output_roll_message(
    ctx: Context<'_>,
    roll_message: String,
    username: String,
    channel: Option<ChannelId>,
) -> Result<(), Error> {
    let colour = crate::common::get_author_colour(ctx).await?;

    let embed = generate_roll_embed(roll_message, &username, colour).await?;

    if let Some(channel) = channel {
        if channel != ctx.channel_id() {
            channel
                .send_message(ctx, CreateMessage::default().embed(embed.clone()))
                .await?;
            ctx.send(
                CreateReply::default()
                    .embed(embed)
                    .ephemeral(true)
                    .content(format!("(sent your roll to <#{channel}>) for you")),
            )
            .await?;

            return Ok(());
        }
    }

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    let result = eval_roll(&dice)?;

    output_roll_message(
        ctx,
        result.message,
        ctx.author()
            .nick_in(
                ctx,
                ctx.guild_id()
                    .expect("Tried to roll in non-guild - TODO remove this issue"),
            )
            .await
            .unwrap_or(ctx.author().name.to_string()),
        None,
    )
    .await?;

    Ok(())
}
