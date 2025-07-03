use rand::seq::SliceRandom;
use std::fs::read_to_string;

use crate::common::Error;
use std::fmt;

pub use phf_macros::phf_map;

#[derive(Debug)]
pub enum DictionaryError {
    FileEmpty,
    NotFound,
    LineNotFound,
}

impl fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DictionaryError::FileEmpty => write!(f, "The dictionary file is empty"),
            DictionaryError::LineNotFound => {
                write!(f, "The line was not found in the dictionary?!")
            }
            DictionaryError::NotFound => {
                write!(f, "A word by those criteria could not be found")
            }
        }
    }
}

impl std::error::Error for DictionaryError {}

// Include a set of default dictionaries at compile time
static SUPER_DICTIONARY: phf::Map<&'static str, &str> = phf_map! {
    "simple" =>
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/dictionary_simple.txt"
        )),
    "full" =>
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/dictionary_full.txt"
        )),
};

fn get_dictionary(dictionary_name: &str) -> Result<String, Error> {
    let content: String;

    if SUPER_DICTIONARY.contains_key(dictionary_name) {
        content = SUPER_DICTIONARY[dictionary_name].to_string();
    } else {
        content = read_to_string(dictionary_name)?;
    }
    if content == "" {
        return Err(Box::new(DictionaryError::FileEmpty));
    }

    Ok(content)
}

pub fn read_random_line(dictionary: &str, length: Option<usize>) -> Result<String, Error> {
    let dictionary_content = get_dictionary(dictionary)?;

    let lines: Vec<&str> = dictionary_content
        .lines()
        .filter(|line| length.map_or(true, |len| line.len() == len))
        .collect();

    if lines.is_empty() {
        return Err(Box::new(DictionaryError::NotFound));
    }

    lines
        .choose(&mut rand::thread_rng())
        .map(|&line| line.to_string())
        .ok_or(Box::new(DictionaryError::LineNotFound))
}
