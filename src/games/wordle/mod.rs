// use crate::common::emojify_char;
use crate::common::emojify_custom;
use crate::common::Context;
use crate::common::Error;
use crate::dictionary;

mod models;

use models::Wordle;

use crate::common::emojify_char;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

lazy_static! {
    static ref CURRENT_WORDLE: Mutex<Option<Wordle>> = Mutex::new(None);
}

async fn format_wordle(wordle: &Wordle, ctx: Context<'_>) -> Result<String, Error> {
    // let guesses: Vec<String> = self.guesses.iter().map(ToString::to_string).collect();

    let mut blank_row: String = "".to_string();

    for _ in wordle.word.chars() {
        blank_row = blank_row + ("⬛ ");
    }

    let mut message: String = "".to_string();

    let guess_count: i32 = wordle
        .guesses
        .len()
        .try_into()
        .expect("Count was negative?");

    for guess in &wordle.guesses {
        let mut i = 0;

        for guess_char in guess.value.chars() {
            let mut format = "{}_dark";

            if wordle.word.contains(guess_char) {
                if wordle.word.chars().nth(i).expect(&format!(
                    "Tried comparing a character that did not exist in a wordle {i}, {}",
                    wordle.word
                )) == guess_char
                {
                    format = ":regional_indicator_{}:" //Char in right position
                } else {
                    format = "{}_yellow";
                }
            }

            message = message + &emojify_char(&guess_char, Some(format), Some(ctx)).await? + " ";

            i += 1;
        }

        message.push('\n');
    }

    for i in 0..wordle.num_of_guesses - guess_count {
        message = message + &blank_row + "\n";
    }

    Ok(format!("{message}"))
}

pub fn generate_wordle(string_length: Option<usize>) -> Result<Wordle, Error> {
    let word = dictionary::read_random_line("simple", Some(string_length.unwrap_or(5)))?;

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
    let mut new_wordle = generate_wordle(string_length)?;

    ctx.say(format!("Debug: The word is: {}", &new_wordle.word))
        .await?;

    new_wordle.guess("Hello")?;
    new_wordle.guess("Testh")?;
    new_wordle.guess("Blahb")?;

    let wordle_formatted = format_wordle(&new_wordle, ctx).await?;

    let mut current_wordle = CURRENT_WORDLE.lock().await;
    *current_wordle = Some(new_wordle);

    let _ = ctx.say(wordle_formatted).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn guess_wordle(ctx: Context<'_>, guess: String) -> Result<(), Error> {
    let emojis = crate::common::get_emojis(&ctx.serenity_context()).await;

    let mut msg: String = "".to_string();

    // for (k, v) in emojis {
    //     println!("{}", &format!("{}: {}", k, v));
    // }

    msg = emojify_custom(ctx, &guess, &"{}_dark").await;

    let _ = ctx.say(msg).await?;
    Ok(())
}
