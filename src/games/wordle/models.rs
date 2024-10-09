use crate::common::Error;
use std::fmt;

#[derive(Debug)]
pub enum WordleError {
    WrongGuessLength,
}

impl fmt::Display for WordleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for WordleError {}

#[derive(Debug)]
pub struct WordleGuess {
    pub value: String,
}

impl WordleGuess {}

#[derive(Debug)]
pub struct Wordle {
    pub word: String,
    pub guesses: Vec<WordleGuess>,
    pub num_of_guesses: i32,
}

impl Wordle {
    pub fn guess(&mut self, guess_value: &str) -> Result<(), Error> {
        if guess_value.len() != self.word.len() {
            return Err(Box::new(WordleError::WrongGuessLength));
        }

        self.guesses.push(WordleGuess {
            value: guess_value.to_string(),
        });

        Ok(())
    }
}
