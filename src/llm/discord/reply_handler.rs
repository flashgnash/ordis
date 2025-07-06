use async_trait::async_trait;
use poise::serenity_prelude::EventHandler;

use crate::{
    common::{fetch_message, fetch_message_chain},
    llm::{self},
};

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

                if crate::llm::contains_badness(
                    &original_message.content,
                    &*super::DISALLOWED_CATEGORIES,
                )
                .await
                .expect("Filter error in event handler")
                {
                    msg.reply(ctx, "Sorry, I'm afraid I can't respond to that")
                        .await
                        .expect("Failure sending message in event handler");
                    return;
                }

                let mut messages = vec![llm::Message {
                    role: llm::Role::system,
                    content: crate::llm::Personality::Ordis.get().to_string(),
                    name: None,
                }];

                let mut message_chain = fetch_message_chain(&ctx, channel_ref, message_ref)
                    .await
                    .unwrap(); //("Fetch message chain failed");
                message_chain.reverse();

                for chain_message in message_chain {
                    let author_name = chain_message.author.name.to_string();
                    if !crate::llm::contains_badness(
                        &chain_message.content,
                        &*super::DISALLOWED_CATEGORIES,
                    )
                    .await
                    .expect("Filter error in event handler")
                    {
                        let role = if chain_message.author.id == ctx.cache.current_user().id {
                            llm::Role::assistant
                        } else {
                            llm::Role::user
                        };

                        messages.push(llm::Message {
                            role,
                            content: format!(
                                "The following message was sent by the user {}. Message: {}",
                                chain_message.author.name,
                                chain_message.content.to_string()
                            ),
                            name: Some(author_name),
                        });
                    } else {
                        messages.push(llm::Message {
                            role: llm::Role::user,
                            content: format!("{}: BZZZZT (it seems static has prevented you from reading this message)",chain_message.author.name).to_string(),
                            name:  Some(author_name)
                        })
                    }
                }

                messages.push(llm::Message {
                    role: llm::Role::assistant,
                    content: original_message.content.to_string(),
                    name: None,
                });

                messages.push(llm::Message {
                    role: llm::Role::assistant,
                    content: msg.content.to_string(),
                    name: None,
                });

                for message in &messages {
                    println!("{}", message.content)
                }

                let response = llm::generate_to_string(None, messages).await.unwrap();

                if let Err(why) = &msg.reply(&ctx.http, response.to_string()).await {
                    println!("Error sending message: {:?}", why);
                }

                return;
            }
        } else if msg.mentions_user_id(ctx.cache.current_user().id) {
            let response;

            if crate::llm::contains_badness(&msg.content, &*super::DISALLOWED_CATEGORIES)
                .await
                .expect("Filter failed in event handler")
            {
                response = "Sorry, I'm afraid I can't respond to that.".to_string();
            } else {
                response =
                    crate::llm::discord::generate_ordis(&msg.content, Some(&msg.author.name), None)
                        .await
                        .expect("LLM call failed in event handler")
                        .choices[0]
                        .message
                        .content
                        .to_string();
            }

            println!("{}", response);

            if let Err(why) = &msg.reply(&ctx.http, response.to_string()).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}
