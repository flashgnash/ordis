use crate::common::HTTP_CLIENT;

use songbird::input::YoutubeDl;

use crate::common::Context;
use crate::common::Error;

#[poise::command(slash_command, prefix_command)]
pub async fn play_music(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let do_search = !url.starts_with("http");

    let http_client = HTTP_CLIENT.clone();

    let handler_lock = super::join_user_channel(ctx).await?;

    let mut handler = handler_lock.lock().await;

    let src = if do_search {
        YoutubeDl::new_search(http_client, url)
    } else {
        YoutubeDl::new(http_client, url)
    };

    let _ = handler.enqueue_input(src.clone().into()).await;

    ctx.reply("Playing ").await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn pause_music(ctx: Context<'_>) -> Result<(), Error> {
    let handler_lock = super::join_user_channel(ctx).await?;
    let handler = handler_lock.lock().await;

    handler.queue().pause()?;

    ctx.reply("Paused").await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn resume_music(ctx: Context<'_>) -> Result<(), Error> {
    let handler_lock = super::join_user_channel(ctx).await?;
    let handler = handler_lock.lock().await;

    handler.queue().resume()?;

    ctx.reply("Resuming... ").await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn stop_music(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let handler_lock = super::join_user_channel(ctx).await?;

    let mut handler = handler_lock.lock().await;

    let _ = handler.stop();

    ctx.reply("Stopped music").await?;

    Ok(())
}
