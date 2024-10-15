
use meval::eval_str;


use dotenv::dotenv;

use poise::async_trait;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::EventHandler;
use poise::serenity_prelude::Ready;
mod common;
use crate::common::Context;
use crate::common::Data;
use crate::common::Error;
use crate::common::fetch_message;
use crate::common::fetch_message_chain;
mod dice;
// use dice::roll;

mod db;

mod voice;
use voice::join_vc;
use voice::music::play_music;
use voice::music::stop_music;
use voice::music::pause_music;
use voice::music::resume_music;
use voice::music::skip_song;

use songbird::SerenityInit;



mod mir;

use mir::stat_puller::pull_stats;
use mir::stat_puller::pull_stat;
use mir::pull_spellsheet;

use mir::level_up;
use mir::create_character;
use mir::get_characters;
use mir::roll;
use mir::delete_character;
use mir::select_character;
use mir::get_spell;
use mir::set_spells;

mod gpt;
use gpt::ask;
use gpt::translate;
use gpt::translate_context;
use gpt::draw;
use rand::prelude::*;

pub struct Handler;

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

        if let Some(ref message_reference) = msg.message_reference {
            let message_ref = message_reference.message_id.unwrap();
            let channel_ref = message_reference.channel_id;

            let original_message = fetch_message(&ctx, channel_ref, message_ref).await.unwrap();

            if ctx.cache.current_user().id == original_message.author.id {
                println!("That's me!");


                let mut messages = vec![            

                        gpt::Message {
                            role: gpt::Role::system,
                            content: "You are Ordis, the helpful AI assistant from the game Warframe. You should take on Ordis's personality when responding to prompts, while still being helpful and accurate".to_string()

                        },
                        // gpt::Message {
                        //     role: gpt::Role::assistant,
                        //     content: original_message.content.to_string()

                        // },

                        // gpt::Message {
                        //     role: gpt::Role::user,
                        //     content:msg.content.to_string(),

                        // }
                    ];


                let mut message_chain = fetch_message_chain(&ctx, channel_ref, message_ref).await.unwrap();//("Fetch message chain failed");
                message_chain.reverse();

                for chain_message in message_chain {

                    // println!("Message content: {}",chain_message.content);
                    // Determine role based on whether the message author is the bot
                    let role = if chain_message.author.id == ctx.cache.current_user().id  {
                        gpt::Role::assistant
                    } else {
                        gpt::Role::user
                    };

                    messages.push(gpt::Message {
                        role,
                        content: chain_message.content.to_string(),
                    });
                }                   

                messages.push(gpt::Message {
                    role: gpt::Role::assistant,
                    content: original_message.content.to_string()
                });

                messages.push(gpt::Message {
                    role: gpt::Role::assistant,
                    content: msg.content.to_string()
                });

                for message in &messages {
                    println!("{}",message.content)
                }


                let response = gpt::generate_to_string("gpt-4o-mini",messages).await.unwrap();


                if let Err(why) = &msg.reply(&ctx.http, response.to_string()).await {
                    println!("Error sending message: {:?}", why);
                }

                return;
            }
        }
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
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
    let user_name = &author.name;

    let mut user = db::users::get_or_create(db_connection, user_id).unwrap();

    let characters = db::characters::get_from_user_id(db_connection,user_id)?;


    for character in characters {
        println!("{}",character.name.expect("Character has no name"));
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
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;


    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ping(),
                calc(), 

                ask(), draw(), translate(),translate_context(),

                pull_stat(), pull_stats(), pull_spellsheet(),
                get_characters(), delete_character(),
                select_character(), create_character(), set_spells(),
                
                get_spell(), level_up(), roll(), 

                join_vc(),
                play_music(),stop_music(),pause_music(),resume_music(),skip_song()
            ],

            ..Default::default()
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
        .register_songbird()        
        .await;
    println!("Starting framework...");
    client.unwrap().start().await.unwrap();
}
