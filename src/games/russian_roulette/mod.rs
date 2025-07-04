use std::collections::HashMap;
use std::time::UNIX_EPOCH;

use crate::common::Context;
use crate::common::Error;

use std::time::{Duration, SystemTime};

use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::Timestamp;
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
        if (roll != bullets_left - 1) {
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

    if let Some(guild_id) = ctx.guild_id() {
        if let Ok(mut member) = guild_id.member(ctx, ctx.author().id).await {
            let unix_now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let timeout =
                Timestamp::from_unix_timestamp(unix_now as i64 + 5 * 60).expect("Valid timestamp");

            let _ = member
                .disable_communication_until_datetime(ctx, timeout)
                .await?;

            println!("Attempted to time out member");
        }
    }

    Ok(())
}

pub fn commands() -> Vec<Command<crate::common::Data, crate::common::Error>> {
    return vec![russian_roulette()];
}
