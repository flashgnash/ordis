use poise::{Command, CreateReply};

use crate::{
    common::{Context, Error},
    llm::{generate, Message, OpenAIResponse, Role},
};

pub async fn translate_internal(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Translating, please wait...*").await?;

    let response = generate_translator(&message, "english", "spanish", None).await?;

    let response_message = &response.choices[0].message.content;

    println!("{}", response_message);

    let message_text = format!("translation of: ``{message}``\n\n{response_message}");

    let reply = CreateReply::default().content(message_text);

    msg.edit(ctx, reply).await?;

    Ok(())
}

pub async fn generate_translator(
    message: &str,
    lang1: &str,
    lang2: &str,
    model: Option<&str>,
) -> Result<OpenAIResponse, Error> {
    let messages = vec![
        Message {
            role: Role::system,
            content: format!("Act as a {lang1}-{lang2} translator. Respond with only an accurate translation and nothing else. Please translate to natural speech in the given language"),
            name: None,
        },

        Message {
            role: Role::user,
            content:message.to_string(),
            name: None,

        }
    ];

    return generate(model, messages).await;
}

#[poise::command(slash_command, prefix_command)]
pub async fn translate(ctx: Context<'_>, message: String) -> Result<(), Error> {
    translate_internal(ctx, message).await?;

    return Ok(());
}
#[poise::command(context_menu_command = "Translate message")]
pub async fn translate_context(
    ctx: Context<'_>,
    #[description = "Message to translate"] msg: crate::serenity::Message,
) -> Result<(), Error> {
    translate_internal(ctx, msg.content).await?;

    Ok(())
}

pub fn commands() -> Vec<Command<crate::common::Data, crate::common::Error>> {
    return vec![translate_context(), translate()];
}
