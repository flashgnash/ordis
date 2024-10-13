pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
use poise::serenity_prelude::Message;

use lazy_static::lazy_static;
use poise::serenity_prelude as serenity;

lazy_static! {
    pub static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub async fn check_admin(
    ctx: Context<'_>,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
) -> bool {
    if let Ok(member) = ctx.http().get_member(guild_id, user_id).await {
        let perms = member
            .permissions
            .unwrap_or_else(|| return serenity::Permissions::empty());

        perms.administrator()
    } else {
        false
    }
}

pub async fn fetch_message_chain(
    ctx: &poise::serenity_prelude::Context,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
    let mut messages = Vec::new();

    // Fetch the initial message
    let mut message = ctx.http.get_message(channel_id, message_id).await?;
    messages.push(message.clone());

    match message.message_reference {
        Some(value) => {
            let res = fetch_message(
                ctx,
                value.channel_id,
                value.message_id.expect("No message ID?!"),
            )
            .await?;

            println!("{}", &message.content);
            messages.push(res.clone());
            let future = fetch_message_chain(ctx, res.channel_id, res.id);

            let replies = Box::pin(future).await?;

            for reply in replies {
                messages.push(reply.clone());
            }
        }
        None => println!("End of message chain"),
    }
    Ok(messages)
}

pub async fn fetch_message_poise<E>(
    ctx: &poise::Context<'_, Data, E>,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<poise::serenity_prelude::Message, Error> {
    Ok(ctx.http().get_message(channel_id, message_id).await?)
}

pub async fn fetch_message(
    ctx: &poise::serenity_prelude::Context,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<poise::serenity_prelude::Message, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ctx.http.get_message(channel_id, message_id).await?)
}

pub fn strip_non_numerical(s: &str) -> String {
    s.chars().filter(|c| c.is_digit(10)).collect()
}

pub fn safe_to_number(s: &str) -> i32 {
    let part_stripped = strip_non_numerical(s);

    if part_stripped.len() == 0 {
        return 0;
    }

    return part_stripped.parse::<i32>().unwrap();
}

// &[i32] is a slice reference (which means it doesn't borrow the variable)
pub fn join_to_string(numbers: &[i32], separator: &str) -> String {
    let s: String = numbers
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(separator);
    return s;
}

pub fn sum_array(arr: &Vec<i32>) -> i32 {
    let mut result = 0;

    for num in arr {
        result = result + num;
    }
    return result;
}
