// use crate::common::emojify_char;
use crate::common::emojify_custom;
use crate::common::emojify_string;
use crate::common::get_emojis;
use crate::common::Context;
use crate::common::Error;

mod models;

use models::Wordle;
use models::WordleGuess;

use crate::common::emojify_char;

use crate::dictionary;

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
        for guess_char in guess.value.chars() {
            let mut format = "{}_dark";

            if wordle.word.contains(guess_char) {
                println!("{} contains {}", wordle.word, guess_char);
                format = ":regional_indicator_{}:";
            }

            message = message + &emojify_char(&guess_char, Some(format), Some(ctx)).await? + " ";
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
    let mut wordle = generate_wordle(string_length)?;

    ctx.say(format!("Debug: The word is: {}", wordle.word))
        .await?;

    wordle.guesses.push(WordleGuess {
        value: "Hello".to_string(),
    });

    wordle.guesses.push(WordleGuess {
        value: "Peace".to_string(),
    });
    wordle.guesses.push(WordleGuess {
        value: "Break".to_string(),
    });
    let wordle_formatted = format_wordle(&wordle, ctx).await?;

    let _ = ctx.say(wordle_formatted).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn guess_wordle(ctx: Context<'_>, guess: String) -> Result<(), Error> {
    let emojis = get_emojis(ctx).await;

    let mut msg: String = "".to_string();

    // for (k, v) in emojis {
    //     println!("{}", &format!("{}: {}", k, v));
    // }

    msg = emojify_custom(ctx, &guess, &"{}_dark").await;

    let _ = ctx.say(msg).await?;
    Ok(())
}
