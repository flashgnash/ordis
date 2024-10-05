use rand::prelude::*;

use crate::common::emojify;
use crate::common::get_emojis;
use crate::common::Context;
use crate::common::Error;

use crate::dictionary;
use std::fmt;

pub struct Wordle {
    pub word: String,
    pub guesses: Vec<WordleGuess>,
    pub num_of_guesses: i32,
}

impl fmt::Display for Wordle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let guesses: Vec<String> = self.guesses.iter().map(ToString::to_string).collect();

        let mut blank_row: String = "".to_string();

        for _ in self.word.chars() {
            blank_row = blank_row + ("⬜");
        }

        let mut message: String = "".to_string();

        let guess_count: i32 = self.guesses.len().try_into().expect("Count was negative?");

        for guess in &self.guesses {
            message = message + &guess.to_string() + "\n";
        }

        for i in 0..self.num_of_guesses - guess_count {
            message = message + &blank_row + "\n";
        }

        write!(f, "{message}")
    }
}

pub struct WordleGuess {
    pub value: String,
}

impl fmt::Display for WordleGuess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", emojify(&self.value))
    }
}

pub fn generate_wordle(string_length: Option<usize>) -> Result<Wordle, Error> {
    let word = dictionary::read_random_line("simple", string_length)?;

    Ok(Wordle {
        word: word.to_string(), //Use to_string to not steal ownership of word
        guesses: Vec::new(),
        num_of_guesses: 5,
    })
}

#[poise::command(slash_command, prefix_command)]
pub async fn get_word(ctx: Context<'_>, string_length: Option<usize>) -> Result<(), Error> {
    let word = dictionary::read_random_line("simple", string_length)?;

    let msg = ctx.say(word).await?;

    Ok(())
}
#[poise::command(slash_command, prefix_command)]
pub async fn get_wordle(ctx: Context<'_>, string_length: Option<usize>) -> Result<(), Error> {
    let mut wordle = generate_wordle(Some(5))?;

    wordle.guesses.push(WordleGuess {
        value: "Hello".to_string(),
    });

    let _ = ctx.say(wordle.to_string()).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn guess_wordle(ctx: Context<'_>, string_length: Option<usize>) -> Result<(), Error> {
    let emojis = get_emojis(ctx, 1242498464605012089).await;

    let mut msg: String = "".to_string();

    for (k, v) in emojis {
        msg = msg + &format!("{}: {}", k, v) + "\n";
    }

    let _ = ctx.say(msg).await?;
    Ok(())
}
