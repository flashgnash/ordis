use std::collections::HashMap;

use crate::common::Context;
use crate::common::Error;

use poise::serenity_prelude::ChannelId;
use poise::Command;
use rand::prelude::*;

use tokio::sync::Mutex;
use tokio::sync::MutexGuard;

use lazy_static::lazy_static;

lazy_static! {
    static ref CHANNEL_ROLLS: Mutex<HashMap<ChannelId, i32>> = Mutex::new(HashMap::new());
}

#[poise::command(slash_command, prefix_command)]
pub async fn russian_roulette(ctx: Context<'_>) -> Result<(), Error> {
    let mut map = CHANNEL_ROLLS.lock().await;

    let count = map.entry(ctx.channel_id()).or_default();
    let bullets_left = 6 - *count;

    println!("bullets left: {bullets_left}");
    if bullets_left > 1 {
        let roll = {
            let mut rng = rand::thread_rng();
            rng.gen_range(1..(bullets_left))
        };
        println!("roll: {roll}");
        if (roll != bullets_left) {
            let count = map
                .entry(ctx.channel_id())
                .and_modify(|c| *c += 1)
                .or_insert(2);

            ctx.say(format!(
                "*click*... ({} bullets left in the chamber)",
                bullets_left - 1
            ))
            .await?;

            return Ok(());
        }
    }

    let count = map
        .entry(ctx.channel_id())
        .and_modify(|c| *c = 0)
        .or_insert(0);

    ctx.say("Bang!").await?;
    Ok(())
}

pub fn commands() -> Vec<Command<crate::common::Data, crate::common::Error>> {
    return vec![russian_roulette()];
}
