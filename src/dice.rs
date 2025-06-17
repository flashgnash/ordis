use poise::serenity_prelude::Colour;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::Embed;
use poise::CreateReply;
use rand::prelude::*;

use crate::common::Context;
use crate::common::Error;

use crate::common::safe_to_number;

use crate::common::join_to_string;
use crate::common::sum_array;

use meval::eval_str;

extern crate regex;
use regex::Regex;
use std::fmt;

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

pub fn roll_one_instance(instance: &str) -> Result<(i32, Vec<i32>), DiceError> {
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

    Ok((sum_array(&dice_rolls), dice_rolls))
}

fn roll_matches(input: &str, pattern: &Regex) -> Result<(String, String), DiceError> {
    let mut result = input.to_string();

    let mut message = "".to_string();

    for mat in pattern.find_iter(input) {
        let (processed, rolls) = roll_one_instance(mat.as_str())?;

        let mat_str = mat.as_str();

        message = format!("{message}\n- {mat_str}: [{}] ", join_to_string(&rolls, ","));
        result = result.replacen(mat_str, &processed.to_string(), 1);
    }
    Ok((result, message))
}

pub fn roll_replace(text: &str) -> Result<(String, String), DiceError> {
    //Change name later this is terrible

    let regex_string = r"\d+d\d+";

    let regex = Regex::new(regex_string).unwrap(); // This regex pattern matches three-letter words

    let (result, message) = roll_matches(&text, &regex)?;

    return Ok((result, message));
}

fn generate_randoms(count: i32, faces: i32) -> Vec<i32> {
    let mut rng = rand::thread_rng();

    let mut rolls: Vec<i32> = vec![];

    for _i in 0..count {
        rolls.push(rng.gen_range(1..faces + 1));
    }

    return rolls;
}

fn pad_string(input: &str, total_len: usize) -> String {
    format!("{:<width$}", input, width = total_len)
}

pub async fn roll_internal(dice: &String) -> Result<(String, f64), Error> {
    let (replaced, messages) = roll_replace(dice)?;

    let calc_result = eval_str(&replaced)?;

    let message = format!("{dice} {messages}\n\n Result: __{calc_result}__");

    let result = (message, calc_result);

    Ok(result)
}

pub async fn output_roll_message(
    ctx: Context<'_>,
    roll: (String, f64),
    username: String,
) -> Result<(), Error> {
    let colour = crate::common::get_author_colour(ctx).await?;

    let embed = generate_roll_embed(roll, username, colour).await?;

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

pub async fn generate_roll_embed(
    roll: (String, f64),
    username: String,
    colour: Colour,
) -> Result<CreateEmbed, Error> {
    let (message, _) = roll;

    let embed = CreateEmbed::default()
        .title(format!("Rolling for {username}..."))
        .colour(colour)
        .description(format!("\n​\n{message}"));

    Ok(embed)
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    let results = roll_internal(&dice).await?;

    output_roll_message(
        ctx,
        results,
        ctx.author()
            .nick_in(
                ctx,
                ctx.guild_id()
                    .expect("Tried to roll in non-guild - TODO remove this issue"),
            )
            .await
            .unwrap_or(ctx.author().name.to_string()),
    )
    .await?;

    Ok(())
}
