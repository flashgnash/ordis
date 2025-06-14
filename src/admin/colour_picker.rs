use crate::common::Context;
use crate::common::Error;

use crate::serenity::async_trait;
use crate::serenity::ChannelId;
use crate::serenity::Colour;
use crate::serenity::EditRole;
use crate::serenity::GuildId;

use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::Call;

use std::sync::Arc;
use tokio::sync::Mutex;

#[poise::command(slash_command, prefix_command)]
pub async fn set_colour(ctx: Context<'_>, name_colour: String) -> Result<(), Error> {
    let role_name = ctx.author().id.to_string();

    let colour = Colour::from(u32::from_str_radix(&name_colour, 16)?);
    let guild_id = ctx.guild_id().ok_or("No guild")?;

    let http = ctx.serenity_context().http.clone();

    let role = guild_id
        .create_role(&http, EditRole::new().name(role_name).colour(colour))
        .await?;

    let user = ctx.author_member().await.ok_or("No member")?;
    user.add_role(&http, role.id).await?;

    Ok(())
}
