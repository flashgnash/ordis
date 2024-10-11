use crate::common::Context;
use crate::common::Error;

use crate::common::HTTP_CLIENT;

use reqwest;

use crate::serenity::ChannelId;
use crate::serenity::GuildId;

use std::fmt;

use songbird::input::YoutubeDl;

#[derive(Debug)]
pub enum VoiceError {
    FailedToAcquireLock,
    FailedToAcquireManager,
    UserNotInVoiceChannel,
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::FailedToAcquireLock => write!(f, "Failed to acquire lock"),
            VoiceError::FailedToAcquireManager => write!(f, "Failed to acquire manager"),
            VoiceError::UserNotInVoiceChannel => write!(f, "User not in voice channel"),
        }
    }
}
impl std::error::Error for VoiceError {}

pub async fn join_internal(
    ctx: Context<'_>,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<(), Error> {
    let manager_option = songbird::get(ctx.serenity_context()).await.clone();

    if let Some(manager) = manager_option {
        if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
            // Attach an event handler to see notifications of all track errors.
            println!("Succesfully joined voice channel!");
        } else {
            return Err(Box::new(VoiceError::FailedToAcquireLock));
        }
    }

    Ok(())
}

pub async fn join_user_channel(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            return Err(Box::new(VoiceError::UserNotInVoiceChannel));
        }
    };

    join_internal(ctx, guild_id, connect_to).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn join_vc(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Attempting to join your channel").await?;

    join_user_channel(ctx).await?;

    ctx.reply("Succesfully joined your channel.").await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn play_music(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let do_search = !url.starts_with("http");

    let guild_id = ctx.guild_id().unwrap();

    let http_client = reqwest::Client::new();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let mut src = if do_search {
            YoutubeDl::new_search(http_client, url)
        } else {
            YoutubeDl::new(http_client, url)
        };
        let _ = handler.play_input(src.clone().into());

        ctx.reply("Playing song").await?;
    } else {
        return Err(Box::new(VoiceError::UserNotInVoiceChannel));
    }

    Ok(())
}
