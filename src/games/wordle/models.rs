use crate::common::emojify_string;
use std::fmt;

pub struct WordleGuess {
    pub value: String,
}
impl WordleGuess {}

pub struct Wordle {
    pub word: String,
    pub guesses: Vec<WordleGuess>,
    pub num_of_guesses: i32,
}
