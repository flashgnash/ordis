use async_trait::async_trait;
use poise::serenity_prelude::EventHandler;

use crate::{common::{fetch_message, fetch_message_chain}, llm};



pub struct ReplyHandler;

#[async_trait]
impl EventHandler for ReplyHandler {
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
                println!("LLM module Received message response");

                let mut messages = vec![            

                        llm::Message {
                            role: llm::Role::system,
                            content: "You are Ordis, the helpful AI assistant from the game Warframe. You should take on Ordis's personality when responding to prompts, while still being helpful and accurate".to_string()

                        },

                    ];


                let mut message_chain = fetch_message_chain(&ctx, channel_ref, message_ref).await.unwrap();//("Fetch message chain failed");
                message_chain.reverse();

                for chain_message in message_chain {
                    let role = if chain_message.author.id == ctx.cache.current_user().id  {
                        llm::Role::assistant
                    } else {
                        llm::Role::user
                    };

                    messages.push(llm::Message {
                        role,
                        content: chain_message.content.to_string(),
                    });
                }                   

                messages.push(llm::Message {
                    role: llm::Role::assistant,
                    content: original_message.content.to_string()
                });

                messages.push(llm::Message {
                    role: llm::Role::assistant,
                    content: msg.content.to_string()
                });

                for message in &messages {
                    println!("{}",message.content)
                }


                let response = llm::generate_to_string(None,messages).await.unwrap();


                if let Err(why) = &msg.reply(&ctx.http, response.to_string()).await {
                    println!("Error sending message: {:?}", why);
                }

                return;
            }
        }
    }
}
