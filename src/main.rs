use poise::serenity_prelude as serenity;
use evalexpr::*;


mod common;
use crate::common::Data;
use crate::common::Error;
use crate::common::Context;

mod dice;
use dice::roll;

mod db;
use db::test;

#[poise::command(slash_command, prefix_command)]
async fn calc(ctx: Context<'_>, formula: String) -> Result<(),Error> {


    let evaluation = eval(&formula)?;
    
    let _ = ctx.say(format!("{} = {}",formula,evaluation)).await?;

    Ok(())
    
}


#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(),Error> {


    let author = &ctx.author();

    let user = db::User {
       id: author.id.0,
       username: author.name.clone(),
       count: None

    };

    let user_deets = test(user)?;
    let _ = ctx.say(format!("{}",user_deets)).await?;

    Ok(())
    
}

#[tokio::main]
async fn main() {

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(),roll(),calc()],
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

    println!("Starting framework...");
    framework.run().await.unwrap();

}
