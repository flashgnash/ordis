use common::ButtonEventSystem;
use meval::eval_str;

mod admin;

use dotenv::dotenv;

use poise::async_trait;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::CreateButton;
use poise::serenity_prelude::CreateSelectMenuOption;
use poise::serenity_prelude::EventHandler;
use poise::serenity_prelude::Ready;

use serde::Serialize;
use tokio::sync::Mutex;

mod common;
use crate::common::Context;
use crate::common::Data;
use crate::common::Error;

mod db;
mod dice;
mod llm;
mod rpg;
mod voice;

use songbird::SerenityInit;

use lazy_static::lazy_static;
use rand::prelude::*;

pub struct Handler;

lazy_static! {
    static ref EVENT_SYSTEM: Mutex<ButtonEventSystem> = {
        let event_system = ButtonEventSystem::new();

        // Register the standalone function as a handler at startup

        Mutex::new(event_system)
    };
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(
        &self,
        ctx: poise::serenity_prelude::Context,
        msg: poise::serenity_prelude::Message,
    ) {
        if msg.author.bot {
            return;
        }

        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn interaction_create(
        &self,
        ctx: poise::serenity_prelude::Context,
        interaction: poise::serenity_prelude::Interaction,
    ) {
        match interaction {
            poise::serenity_prelude::Interaction::Component(component) => {
                let event_system = EVENT_SYSTEM.lock().await;

                component
                    .create_response(&ctx, serenity::CreateInteractionResponse::Acknowledge)
                    .await
                    .expect("Huh");

                if let poise::serenity_prelude::ComponentInteractionDataKind::StringSelect {
                    values,
                } = &component.data.kind
                {
                    if let Some(selected) = values.get(0) {
                        event_system.emit_event(&ctx, &component, selected).await;
                        // Select menu event handler needs to use dropdown value not id
                    }
                } else {
                    event_system
                        .emit_event(&ctx, &component, &component.data.custom_id)
                        .await;
                }

                // component.channel_id.send_message(&ctx,CreateMessage::default().content(format!("Test "))).await.expect("Huh");
            }
            _ => {}
        }
    }

    async fn ready(&self, _ctx: poise::serenity_prelude::Context, _ready: Ready) {
        println!("Bot is connected!");
    }

    // You can add other event methods here as needed
}

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

#[derive(Serialize)]
struct Callback<T>
where
    T: Serialize,
{
    name: String,
    params: T,
}

fn create_button_with_callback<T, P>(
    text: &str,
    callback_params: &P,
    button_style: ButtonStyle,
) -> Result<CreateButton, Error>
where
    P: Serialize,
{
    let json = create_callback::<T, P>(callback_params)?;
    Ok(CreateButton::new(json).label(text).style(button_style))
}

fn create_select_option_with_callback<T, P>(
    text: &str,
    callback_params: &P,
) -> Result<CreateSelectMenuOption, Error>
where
    P: Serialize,
{
    let json = create_callback::<T, P>(callback_params)?;
    Ok(CreateSelectMenuOption::new(text, json).label(text))
}

fn create_callback<T, P>(callback_params: &P) -> Result<String, Error>
where
    P: Serialize,
{
    let callback_name = std::any::type_name::<T>()
        .split("::")
        .last()
        .unwrap()
        .to_string();
    // let params = serde_json::to_string(&callback);

    let callback_serializable = Callback {
        name: callback_name,
        params: callback_params,
    };

    let json = serde_json::to_string(&callback_serializable)?;

    Ok(json)
    // Ok(CreateButton::new(json).label(text).style(button_style))
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

    let user_id = author.id.get();

    let mut user = db::users::get_or_create(db_connection, user_id).unwrap();

    let characters = db::characters::get_from_user_id(db_connection, user_id)?;

    for character in characters {
        println!("{}", character.name.expect("Character has no name"));
    }

    user.count = Some(user.count.unwrap_or(0) + 1);
    let _ = db::users::update(db_connection, &user);
    let count;

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
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let mut event_system = EVENT_SYSTEM.lock().await;

    rpg::mir::register_events(&mut event_system);

    // crate::rpg::register_events(&mut event_system);

    drop(event_system);

    // event_system.register_handler("test_event",
    //     |ctx: &poise::serenity_prelude::Context, params: &ButtonParams| {
    //         if let Some(Value::String(val)) = params.get("key") {
    //             println!("Event received with param: {}", val);
    //         }
    //     });

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), calc()]
                .into_iter()
                .chain(llm::discord::commands())
                .chain(llm::discord::translator::commands())
                .chain(voice::music::commands())
                .chain(voice::commands())
                .chain(admin::commands())
                .chain(rpg::mir::commands())
                .collect::<Vec<_>>(),

            ..Default::default() // TODO make this configurable via environment variables
                                 // IE have a list of module names to import, and a dictionary in main it matches them against
                                 // This way I can disable the RPG commands in Ordis and the admin commands in Sentient Bob
                                 // while sharing the same codebase
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(Handler)
        .event_handler(crate::admin::auto_threads::Handler)
        .event_handler(crate::admin::auto_react::Handler)
        .event_handler(crate::llm::discord::reply_handler::ReplyHandler)
        .register_songbird()
        .await;
    println!("Starting framework...");
    client.unwrap().start().await.unwrap();
}
