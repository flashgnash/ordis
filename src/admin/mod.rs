use poise::Command;

pub mod auto_react;
pub mod auto_threads;
pub mod colour_picker;
pub mod nickname;

pub fn commands() -> Vec<Command<crate::common::Data, crate::common::Error>> {
    return vec![colour_picker::set_colour(), nickname::set_nick()];
}
