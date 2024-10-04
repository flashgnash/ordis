use rand::prelude::*;

use crate::common::Context;
use crate::common::Error;

use crate::dictionary;

#[poise::command(slash_command, prefix_command)]
pub async fn get_word(ctx: Context<'_>, string_length: Option<usize>) -> Result<(), Error> {
    let word = dictionary::read_random_line("simple", string_length)?;

    let msg = ctx.say(word).await?;

    Ok(())
}
#[poise::command(slash_command, prefix_command)]
pub async fn get_wordle(ctx: Context<'_>, string_length: Option<usize>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn guess_wordle(ctx: Context<'_>, string_length: Option<usize>) -> Result<(), Error> {
    Ok(())
}
