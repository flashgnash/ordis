use poise::serenity_prelude as serenity;

mod common;
use crate::common::Data;
use crate::common::Error;
use crate::common::Context;

mod dice;
use dice::roll;

mod db;
use db::test;


#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(),Error> {

    ctx.say(format!("Testing {}",ctx.author().id)).await?;

    test();

    Ok(())
    
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(),roll()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
