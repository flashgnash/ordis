use poise::serenity_prelude as serenity;
use rand::prelude::*;

mod common;
use crate::common::Data;
use crate::common::Error;
use crate::common::Context;

mod dice;
use dice::roll;


/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}








#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(),Error> {

    ctx.say("Testing").await?;
    Ok(())
    
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(),ping(),roll()],
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
