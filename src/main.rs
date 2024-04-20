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


use rand::prelude::*;


#[poise::command(slash_command, prefix_command)]
async fn calc(ctx: Context<'_>, formula: String) -> Result<(),Error> {


    let evaluation = eval(&formula)?;
    
    let _ = ctx.say(format!("{} = {}",formula,evaluation)).await?;

    Ok(())
    
}


fn random_number(num1: i32, num2:i32) -> i32 {

   let mut rng = rand::thread_rng();
   return rng.gen_range(1..num2);

}
fn get_random(vec: &Vec<&str>) -> String {
    let count = vec.len() as i32;

    let index = random_number(0,count) as usize;

    return vec[index].to_string();

}



#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(),Error> {


    let quotes : Vec<&str> = vec![
        "Operator? Ordis wonders... what are you thinking about?",
        "Operator, I've run diagnostic regressions. All systems nominal. You don't need to thank me.",
        "Ordis is hap - angry. Hmm, I may require maintenance after all.",
        "Operator, are you enjoying the view?",
        "Do you remember the Old War, Operator? Ordis seems to have... misplaced those memories.",
        "Operator, the system needs you. Will you begin another mission?",
        "I've been thinking, Operator... I thought you'd want to know.",
        "Operator! Did you hear that? It said-- Cosmic background radiation is a riot!",
        "Operator, were you visualizing a bloody battle? -Me too!",
        "Stand by while I analyze the intelligence profile of the Grineer. Error, not a number! Did the Operator enjoy this witticism?",
        "Everything in Ordis, Operator? Is that a pun?! Hmm.... I will attempt to bypass this fault.",
        "Ordis has been counting stars, Operator. All accounted for."

    ];

    let quote = get_random(&quotes);


    let author = &ctx.author();

    let user = db::User {
       id: author.id.0,
       username: author.name.clone(),
       count: None

    };

    let user_deets = test(user)?;
    let _ = ctx.say(format!("{}\n{}",quote,user_deets)).await?;

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
