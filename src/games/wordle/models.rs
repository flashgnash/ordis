use crate::common::emojify_string;
use std::fmt;

pub struct WordleGuess {
    pub value: String,
}
impl WordleGuess {}

impl fmt::Display for WordleGuess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", emojify_string(&self.value))
    }
}
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
            blank_row = blank_row + ("⬜ ");
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
