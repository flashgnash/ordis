use crate::common::HTTP_CLIENT;

use poise::Command;
use songbird::input::Compose;
use songbird::input::YoutubeDl;

use crate::common::Context;
use crate::common::Error;
use poise::CreateReply;

#[poise::command(slash_command, prefix_command)]
pub async fn play_music(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let do_search = !url.starts_with("http");

    let http_client = HTTP_CLIENT.clone();

    let handler_lock = super::join_user_channel(ctx).await?;

    let mut handler = handler_lock.lock().await;

    let mut src = if do_search {
        YoutubeDl::new_search(http_client, url)
    } else {
        YoutubeDl::new(http_client, url)
    };

    let _ = handler.enqueue_input(src.clone().into()).await;

    let track_name = src.aux_metadata().await?.source_url;

    let queue_length = handler.queue().len();

    msg.edit(
        ctx,
        CreateReply::default().content(format!(
            "Adding {} to the queue in position {}",
            track_name.unwrap_or("{no track url}".to_string()),
            queue_length
        )),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn skip_song(ctx: Context<'_>) -> Result<(), Error> {
    let handler_lock = super::join_user_channel(ctx).await?;
    let handler = handler_lock.lock().await;

    // ctx.reply(format!("Skipping {}", handler.queue().current()).await?)
    //     .await?;

    handler.queue().skip()?;

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
pub async fn stop_music(ctx: Context<'_>) -> Result<(), Error> {
    let handler_lock = super::join_user_channel(ctx).await?;

    let handler = handler_lock.lock().await;

    handler.queue().stop();

    ctx.reply("Stopped music").await?;

    Ok(())
}

pub fn commands() -> Vec<Command<crate::common::Data, crate::common::Error>> {
    return vec![
        play_music(),
        stop_music(),
        pause_music(),
        resume_music(),
        skip_song(),
    ];
}
