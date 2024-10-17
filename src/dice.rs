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

    let mut message = format!("- ``{input}``").to_string();

    for mat in pattern.find_iter(input) {
        let (processed, rolls) = roll_one_instance(mat.as_str())?;

        let mat_str = mat.as_str();

        message = format!("{message} {mat_str}: [{}] ", join_to_string(&rolls, ","));
        result = result.replacen(mat_str, &processed.to_string(), 1);
    }
    Ok((result, message))
}

pub fn roll_replace(text: &str) -> Result<(String, String), DiceError> {
    //Change name later this is terrible

    let regex_string = r"\d+d\d+";

    let regex = Regex::new(regex_string).unwrap(); // This regex pattern matches three-letter words

    // let result = regex.replace_all(&original, |caps: &Captures| {
    //     let cap = caps.get(0)?.as_str()
    //     roll_one_instance(cap)?.to_string()
    // });

    let (result, message) = roll_matches(&text, &regex)?;

    return Ok((result, message));
}

fn generate_randoms(count: i32, faces: i32) -> Vec<i32> {
    let mut rng = rand::thread_rng();

    let mut rolls: Vec<i32> = vec![];

    for _i in 0..count {
        rolls.push(rng.gen_range(1..faces));
    }

    return rolls;
}

fn pad_string(input: &str, total_len: usize) -> String {
    format!("{:<width$}", input, width = total_len)
}

pub async fn roll_internal(dice: &String) -> Result<Vec<(String, f64)>, Error> {
    let instances = dice.split(',');

    let mut result: Vec<(String, f64)> = vec![];

    for instance in instances {
        let (replaced, messages) = roll_replace(&instance)?;
        let calc_result = eval_str(&replaced)?;

        let message = format!("{} = {} = __{}__", &messages, &replaced, &calc_result);

        result.push((message, calc_result));
    }

    Ok(result)
}

pub async fn output_roll_messages(
    ctx: Context<'_>,
    rolls: Vec<(String, f64)>,
    username: String,
) -> Result<(), Error> {
    let mut longest_line = 0;
    let mut message_lines: Vec<String> = vec![];
    let mut grand_total = 0.0;

    for (message, calc_result) in rolls {
        if message.len() > longest_line {
            longest_line = message.len();
        }

        grand_total = grand_total + calc_result;

        message_lines.push(message)
    }

    let message = message_lines.join("\n");

    let underline = format!("__{}__", pad_string("", longest_line - 8));
    ctx.say(format!(
        "\n**Rolling for {username}...**\n\n{message}\n{underline}\nTotal: {grand_total}"
    ))
    .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    let results = roll_internal(&dice).await?;

    output_roll_messages(
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
