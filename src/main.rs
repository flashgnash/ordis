

use std::any::type_name;

use common::ButtonEventSystem;
use common::ButtonParams;
use meval::eval_str;

mod admin;
use admin::colour_picker::set_colour;
use admin::nickname::set_nick;

use dotenv::dotenv;

use poise::async_trait;
use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::CreateActionRow;
use poise::serenity_prelude::CreateButton;
use poise::serenity_prelude::CreateMessage;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::EventHandler;
use poise::serenity_prelude::Ready;
use poise::CreateReply;

use serde::Serialize;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;


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

mod elastic;

mod rpg;

use rpg::mir::pull_stats;
use rpg::mir::pull_stat;
use rpg::mir::pull_spellsheet;

use rpg::mir::get_mana;
use rpg::mir::set_mana;
use rpg::mir::mod_mana;
use rpg::mir::add_mana;
use rpg::mir::sub_mana;


use rpg::mir::status;
use rpg::mir::status_admin;

use rpg::mir::level_up;
use rpg::mir::create_character;
use rpg::mir::get_characters;
use rpg::mir::roll;
use rpg::mir::delete_character;
use rpg::mir::select_character;


use rpg::mir::cast_spell;
use rpg::mir::set_spells;
use rpg::mir::list_spells;
use rpg::mir::end_turn;

mod gpt;
use gpt::ask;
use gpt::translate;
use gpt::translate_context;
use gpt::draw;
use rand::prelude::*;
use lazy_static::lazy_static;

pub struct Handler;



fn register_events(event_system: &mut MutexGuard<ButtonEventSystem>) {
    event_system.register_handler(TestEvent);
}



// Example event
pub struct TestEvent;

#[derive(Serialize)]
pub struct TestEventParams {
    key: String,
}

impl TestEvent {
    fn create_button(text: &str, params: &TestEventParams, button_style: ButtonStyle) -> Result<CreateButton,Error> {
        return create_button_with_callback::<Self,TestEventParams>(text,params,button_style);
    }
}

#[async_trait]
impl common::EventHandlerTrait for TestEvent {
    async fn run(&self,ctx: &poise::serenity_prelude::Context,interaction: &poise::serenity_prelude::ComponentInteraction,params: &ButtonParams) {
        if let Some(Value::String(val)) = params.get("key") {
            println!("Event received with param: {}", val);
            interaction.channel_id.send_message(ctx,CreateMessage::default().content(format!("Event received with param: {} from user {}", val, interaction.user.name))).await.expect("AAA");
        }
    }
}
// Example event end


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

    async fn interaction_create(
        &self,
        ctx: poise::serenity_prelude::Context,
        interaction: poise::serenity_prelude::Interaction,
    ) {

    match interaction {
        poise::serenity_prelude::Interaction::Component(component) => {

            let event_system = EVENT_SYSTEM.lock().await;

            
            component.create_response(&ctx,
                serenity::CreateInteractionResponse::Acknowledge
            ).await.expect("Huh");

            event_system.emit_event(&ctx,&component,&component.data.custom_id).await;    
            

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
T: Serialize
{
    name: String,
    params: T
}

fn create_button_with_callback<T,P>(text:&str,callback_params: &P,button_style:ButtonStyle) -> Result<CreateButton,Error>
where P: Serialize
 
{

    let callback_name = std::any::type_name::<T>()
        .split("::")
        .last()
        .unwrap()
        .to_string();
    // let params = serde_json::to_string(&callback);

    let callback_serializable = Callback {
        name: callback_name,
        params: callback_params
    };


    let json = serde_json::to_string(&callback_serializable)?;

    Ok(CreateButton::new(json).label(text).style(button_style))
    
}


#[poise::command(slash_command, prefix_command)]
async fn button_test(ctx: Context<'_>) -> Result<(), Error> {
    // Define buttons as variables

    let rows = vec![
        CreateActionRow::Buttons(vec![

            TestEvent::create_button("Click me!",
                &TestEventParams {
                    key: "Hello world!".to_string()
                },
                ButtonStyle::Primary
            )?,

            TestEvent::create_button("Click me!",
                &TestEventParams {
                    key: "Testing world!".to_string()
                },
                
                ButtonStyle::Primary
            )?
        ])
    ];
    
    let msg = CreateReply::default()
        .content("Hello")
        .components(rows);

    // Send message with action row
    ctx.send( msg ).await?;

    Ok(())

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
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_PRESENCES;


    let mut event_system = EVENT_SYSTEM.lock().await;

    register_events(&mut event_system);
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
            commands: vec![
                ping(), button_test(),
                calc(), 

                ask(), draw(), translate(),translate_context(),

                pull_stat(), pull_stats(), pull_spellsheet(),
                get_mana(), set_mana(), mod_mana(), add_mana(), sub_mana(),
                status(),status_admin(),
                get_characters(), delete_character(),
                select_character(), create_character(), set_spells(),
                
                cast_spell(), list_spells(), level_up(), roll(), end_turn(),

                join_vc(),
                play_music(),stop_music(),pause_music(),resume_music(),skip_song(),


                set_colour(), set_nick()
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
