use std::fs::OpenOptions;
use std::io::Write;

use chrono::Utc;
pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
use std::{collections::HashMap, future::Future};

use poise::serenity_prelude::{Colour, Message};
use serde::Deserialize;
use serde_json::{from_str, Value};

use crate::db;
use diesel::sqlite::SqliteConnection;

use lazy_static::lazy_static;
use poise::serenity_prelude as serenity;

use async_trait::async_trait;
use sha2::{Digest, Sha256};

lazy_static! {
    pub static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

#[async_trait]
pub trait EventHandlerTrait: Send + Sync {
    async fn run(
        &self,
        ctx: &poise::serenity_prelude::Context,
        interaction: &poise::serenity_prelude::ComponentInteraction,
        params: &ButtonParams,
    );
}

pub type ButtonParams = HashMap<String, Value>;

type ButtonHandler = Box<
    dyn Fn(
            &poise::serenity_prelude::Context,
            &poise::serenity_prelude::ComponentInteraction,
            &ButtonParams,
        ) -> Box<dyn Future<Output = ()>>
        + Send
        + Sync,
>;

pub async fn log_if_failed_async<T, E: std::fmt::Debug>(
    fut: impl std::future::Future<Output = Result<T, E>>,
) -> Option<T> {
    match fut.await {
        Ok(val) => Some(val),
        Err(e) => {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
            let msg = format!("[{} UTC] Error: {:?}\n", timestamp, e);
            eprint!("{}", msg);

            if let Ok(mut file) = OpenOptions::new()
                .append(true)
                .create(true)
                .open("errors.log")
            {
                let _ = file.write_all(msg.as_bytes());
            }

            None
        }
    }
}
pub struct ButtonEventSystem {
    handlers: HashMap<String, Vec<Box<dyn EventHandlerTrait>>>,
}

#[derive(Deserialize)]
pub struct ButtonEvent {
    name: String,
    params: ButtonParams,
}

impl ButtonEventSystem {
    pub fn new() -> Self {
        ButtonEventSystem {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<T>(&mut self, handler: T)
    where
        T: EventHandlerTrait + 'static,
    {
        let event_name = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap()
            .to_string();

        println!("Registered handler {event_name}");
        self.handlers
            .entry(event_name)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    pub async fn emit_event(
        &self,
        ctx: &poise::serenity_prelude::Context,
        interaction: &poise::serenity_prelude::ComponentInteraction,
        json: &str,
    ) {
        println!("Emitting event");
        let event: ButtonEvent = match from_str(json) {
            Ok(event) => event,
            Err(err) => {
                eprintln!("Failed to parse event JSON: {}", err);
                return;
            }
        };

        if let Some(handlers) = self.handlers.get(&event.name) {
            for handler in handlers {
                handler.run(ctx, interaction, &event.params).await;
            }
        }
    }
}

pub fn uid_to_rgb(uid: u64) -> (u8, u8, u8) {
    let r = (uid & 0xFF) as u8;
    let g = ((uid >> 8) & 0xFF) as u8;
    let b = ((uid >> 16) & 0xFF) as u8;
    (r, g, b)
}

pub fn username_to_rgb(s: &str) -> (u8, u8, u8) {
    let uid = s
        .as_bytes()
        .iter()
        .fold(0u64, |acc, &b| (acc << 8) | b as u64);
    let r = ((uid & 0xFF) % 128 + 128) as u8; // Brightness adjustment
    let g = (((uid >> 8) & 0xFF) % 128 + 128) as u8; // Brightness adjustment
    let b = (((uid >> 16) & 0xFF) % 128 + 128) as u8; // Brightness adjustment
    (r, g, b)
}

pub async fn get_author_role_colour(ctx: Context<'_>) -> Result<Option<Colour>, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let member = guild_id.member(&ctx, ctx.author().id).await?;

        if let Some(mut roles) = member.roles(ctx) {
            roles.sort_by_key(|r| r.position);
            roles.reverse();
            for role in roles {
                if role.colour.hex() != "000000" {
                    return Ok(Some(role.colour));
                }
            }
        }
    }
    Ok(None)
}

pub async fn get_author_colour(ctx: Context<'_>) -> Result<Colour, Error> {
    if let Some(colour) = get_author_role_colour(ctx).await? {
        Ok(colour)
    } else {
        let (r, g, b) = uid_to_rgb(ctx.author().id.try_into()?);
        Ok(Colour::from_rgb(r, g, b))
    }
}

pub fn is_author_on_mobile(ctx: &Context<'_>) -> bool {
    if let Some(guild) = ctx.guild() {
        let presence = guild.presences.get(&ctx.author().id);
        if let Some(presence) = presence {
            if let Some(client_status) = &presence.client_status {
                if let Some(mobile_status) = client_status.mobile {
                    if mobile_status == poise::serenity_prelude::OnlineStatus::Online {
                        println!("User detected on mobile");
                        return true;
                    }
                }
            }
        }
    }
    return false;
}

pub fn draw_bar(
    current: i32,
    max: i32,
    length: usize,
    foreground: &str,
    background: &str,
) -> String {
    let fraction = current as f32 / max as f32;

    let current_length = (fraction as f32 * length as f32).round() as usize;

    return foreground.repeat(current_length)
        + &background.repeat(length - current_length.clamp(0, length));
}

pub async fn get_user(
    ctx: &Context<'_>,
    db_connection: &mut SqliteConnection,
) -> Result<db::models::User, Error> {
    let author = &ctx.author();

    let user_id = author.id.get();

    Ok(db::users::get_or_create(db_connection, user_id)?)
}

pub fn hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();

    format!("{:x}", result)
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
