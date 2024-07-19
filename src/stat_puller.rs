use rand::prelude::*;

use crate::common::Context;
use crate::common::Error;

use crate::common::safe_to_number;

use crate::common::join_to_string;
use crate::common::sum_array;

extern crate regex;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

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

pub fn pull_stat(instance: &str) -> Result<(i32, Vec<i32>), DiceError> {
    return Ok(0);
}

fn roll_matches(input: &str, pattern: &Regex) -> Result<(String, String), DiceError> {
    let mut result = input.to_string();
    let all_rolls: HashMap<String, Vec<i32>> = HashMap::new();

    let mut message = format!("- ``{input}``").to_string();

    for mat in pattern.find_iter(input) {
        let (processed, rolls) = roll_one_instance(mat.as_str())?;

        let mat_str = mat.as_str();

        message = format!("{message} {mat_str}: [{}] ", join_to_string(&rolls, ","));
        result = result.replacen(mat_str, &processed.to_string(), 1);
    }
    Ok((result, message))
}

fn roll_replace(text: &str) -> Result<(String, String), DiceError> {
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

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    let instances = dice.split(',');

    let mut result: Vec<String> = vec![];

    let mut grand_total = 0.0;

    let mut longest_line = 0;

    for instance in instances {
        let (replaced, messages) = roll_replace(&instance)?;
        let calc_result = eval_str(&replaced)?;

        let message = format!("{} = {} = __{}__", &messages, &replaced, &calc_result);

        let total = calc_result;

        if message.len() > longest_line {
            longest_line = message.len();
        }

        grand_total = grand_total + total;
        result.push(message);
    }

    let message = result.join("\n");

    let underline = format!("__{}__", pad_string("", longest_line - 8));
    ctx.say(format!(
        "\n**Rolling...**\n\n{}\n{}\nTotal: {}",
        message, underline, grand_total
    ))
    .await?;

    Ok(())
}
