pub mod reply_handler;
pub mod translator;

use poise::{Command, CreateReply};

use crate::{
    common::{Context, Error},
    llm::{generate_agent, generate_image, BadKind, OpenAIResponse, Personality},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref DISALLOWED_CATEGORIES: Vec<BadKind> = vec![
        BadKind::Sexual,
        BadKind::SexualMinors,
        BadKind::HarassmentThreatening,
        BadKind::Hate,
        BadKind::IllicitViolent,
        BadKind::SelfHarm,
        BadKind::SelfHarmIntent,
        BadKind::SelfHarmInstructions,
        BadKind::Harassment
    ];
}

pub async fn generate_ordis(message: &str, model: Option<&str>) -> Result<OpenAIResponse, Error> {
    Ok(generate_agent(message, model, Personality::Ordis.get()).await?)
}

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response_message: String;

    if crate::llm::contains_badness(&message, &*DISALLOWED_CATEGORIES).await? {
        response_message = "Sorry, I'm afraid I can't respond to that".to_string();
    } else {
        let response = generate_ordis(&message, None).await?;

        response_message = response.choices[0].message.content.to_string();

        println!("{}", response_message);
    }

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(slash_command, prefix_command)]
pub async fn draw(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response_message;

    if crate::llm::contains_badness(&message, &*DISALLOWED_CATEGORIES).await? {
        response_message = "Sorry, I'm afraid I can't respond to that.".to_string();
    } else {
        response_message = generate_image(&message).await?;
    }

    // println!("Generated image URL: {}", response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}

pub fn commands() -> Vec<Command<crate::common::Data, crate::common::Error>> {
    return vec![draw(), ask()];
}

pub struct MessageReplyHandler;
