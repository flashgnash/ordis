use meval::eval_str;
use poise::serenity_prelude as serenity;

mod common;
use crate::common::Context;
use crate::common::Data;
use crate::common::Error;

mod dice;
use dice::roll;

mod db;
use crate::db::models::User;

mod gpt;
use gpt::ask;
use gpt::translate;

use rand::prelude::*;

#[poise::command(slash_command, prefix_command)]
async fn calc(ctx: Context<'_>, formula: String) -> Result<(), Error> {
    let evaluation = eval_str(&formula)?;

    let _ = ctx.say(format!("{formula} = {evaluation}")).await?;

    Ok(())
}

fn random_number(_num1: i32, num2: i32) -> i32 {
    let mut rng = rand::thread_rng();
    return rng.gen_range(1..num2);
}
fn get_random(vec: &Vec<&str>) -> String {
    let count = vec.len() as i32;

    let index = random_number(0, count) as usize;

    return vec[index].to_string();
}

#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
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

    let db_connection = &mut db::establish_connection();

    let user_id = author.id.0;
    let user_name = &author.name;

    let mut userResult = db::users::get(db_connection, user_id);
    let mut user: User;

    match userResult {
        Ok(v) => {
            user = v;
            user.count = Some(user.count.unwrap_or(0) + 1);
            db::users::update(db_connection, &user);
        }
        Err(e) => {
            println!("User not found ({})", e);

            let new_user = User {
                id: user_id.to_string(),
                username: Some(user_name.to_string()),
                count: Some(1),
            };
            db::users::create(db_connection, &new_user);
            user = new_user;
        }
    }

    let mut count = 0;

    match user.count {
        Some(v) => {
            count = v;
        }
        None => {
            count = 0;
        }
    }

    let _ = ctx.say(format!("{quote} {count}")).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), roll(), calc(), ask(), translate()],
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
