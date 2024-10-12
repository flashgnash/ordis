pub mod music;

use crate::common::Context;
use crate::common::Error;

use crate::serenity::async_trait;
use crate::serenity::ChannelId;
use crate::serenity::GuildId;

use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::Call;

use std::sync::Arc;
use tokio::sync::Mutex;

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

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

pub async fn join_internal(
    ctx: Context<'_>,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<Arc<Mutex<Call>>, Error> {
    let manager_option = songbird::get(ctx.serenity_context()).await.clone();

    if let Some(manager) = manager_option {
        if let Ok(handler_lock) = manager.join(guild_id, channel_id).await {
            // Attach an event handler to see notifications of all track errors.
            println!("Succesfully joined voice channel!");

            let mut handler = handler_lock.lock().await;
            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);

            return Ok(handler_lock.clone());
        } else {
            return Err(Box::new(VoiceError::FailedToAcquireLock));
        }
    } else {
        return Err(Box::new(VoiceError::FailedToAcquireManager));
    }
}

pub async fn join_user_channel(ctx: Context<'_>) -> Result<Arc<Mutex<Call>>, Error> {
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

    Ok(join_internal(ctx, guild_id, connect_to).await?)
}

#[poise::command(slash_command, prefix_command)]
pub async fn join_vc(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Attempting to join your channel").await?;

    join_user_channel(ctx).await?;

    ctx.reply("Succesfully joined your channel.").await?;

    Ok(())
}
