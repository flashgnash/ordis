pub mod reply_handler;
pub mod translator;

use poise::{Command, CreateReply};

use crate::{
    common::{Context, Error},
    llm::{generate_agent, generate_image, OpenAIResponse, Personality},
};

pub async fn generate_ordis(message: &str, model: Option<&str>) -> Result<OpenAIResponse, Error> {
    Ok(generate_agent(message, model, Personality::Ordis.get()).await?)
}

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response = generate_ordis(&message, None).await?;

    let response_message = &response.choices[0].message.content;

    println!("{}", response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(slash_command, prefix_command)]
pub async fn draw(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response_message = generate_image(&message).await?;

    println!("Generated image URL: {}", response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}

pub fn commands() -> Vec<Command<crate::common::Data, crate::common::Error>> {
    return vec![draw(), ask()];
}

pub struct MessageReplyHandler;
