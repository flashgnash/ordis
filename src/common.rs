pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
use std::{borrow::Cow, collections::HashMap};

use poise::serenity_prelude::{
    Colour, Emoji, GuildChannel, GuildId, Message, PartialMember, Permissions, UserId,
};
use serde::Deserialize;
use serde_json::{from_str, Value};

use crate::db;
use diesel::sqlite::SqliteConnection;

use poise::serenity_prelude as serenity;

use lazy_static::lazy_static;

use std::fmt;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum EmojiError {
    NotFound,
}

impl fmt::Display for EmojiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for EmojiError {}

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
                eprintln!("Failed to parse event JSON: {}\n\n{}", err, json);
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

pub fn get_channel_tags(channel: &GuildChannel) -> HashMap<String, Vec<String>> {
    if let Some(topic) = &channel.topic {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        for line in topic.lines() {
            let mut parts = line.trim().splitn(2, ' ');
            let key = parts.next().unwrap().trim_start_matches('-').to_string();
            let values = parts
                .next()
                .map(|v| v.split(',').map(str::to_string).collect())
                .unwrap_or_else(Vec::new);
            map.insert(key, values);
        }

        map;
    }

    HashMap::new()
}

pub async fn get_author_perms(ctx: Context<'_>) -> Option<Permissions> {
    if let Some(guild) = ctx.partial_guild().await {
        if let Some(channel) = ctx.guild_channel().await {
            if let Some(member) = ctx.author_member().await {
                // let partial_member = guild.member(ctx, ctx.author()).await?;

                let partial: PartialMember = match member {
                    Cow::Owned(member) => member.into(),
                    Cow::Borrowed(member) => member.clone().into(),
                };

                return Some(guild.partial_member_permissions_in(
                    &channel,
                    ctx.author().id,
                    &partial,
                ));
            }
        }
    }
    None
}

pub fn uid_to_rgb(uid: u64) -> (u8, u8, u8) {
    let r = (uid & 0xFF) as u8;
    let g = ((uid >> 8) & 0xFF) as u8;
    let b = ((uid >> 16) & 0xFF) as u8;
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

pub async fn get_user_colour(
    ctx: &poise::serenity_prelude::Context,
    guild_id: Option<GuildId>,
    user_id: UserId,
) -> Result<Colour, Error> {
    if let Some(guild_id) = guild_id {
        let member = guild_id.member(&ctx, user_id).await?;

        if let Some(mut roles) = member.roles(ctx) {
            roles.sort_by_key(|r| r.position);
            roles.reverse();
            for role in roles {
                if role.colour.hex() != "000000" {
                    return Ok(role.colour);
                }
            }
        }
    }
    return Ok(Colour::default());
}

pub async fn get_author_colour(ctx: Context<'_>) -> Result<Colour, Error> {
    if let Some(colour) = get_author_role_colour(ctx).await? {
        Ok(colour)
    } else {
        let (r, g, b) = uid_to_rgb(ctx.author().id.try_into()?);
        Ok(Colour::from_rgb(r, g, b))
    }
}

#[allow(dead_code)] // I might be using this soon
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

#[allow(dead_code)]
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

lazy_static! {
    static ref EMOJI_CACHE: Mutex<HashMap<String, Emoji>> = Mutex::new(HashMap::new());
}

pub async fn refresh_emojis(ctx: &poise::serenity_prelude::Context) {
    println!("Refreshing emoji cache");

    let guild_ids: Vec<GuildId> = if let Ok(guilds_str) = std::env::var("EMOJI_GUILDS") {
        guilds_str
            .split(',')
            .map(|s| GuildId::from(safe_to_u64(s)))
            .collect()
    } else {
        let ids = ctx.cache.guilds();
        if ids.is_empty() {
            println!("No EMOJI_GUILDS and no cached guilds. Emoji cache empty.");
            return;
        }
        ids.into_iter().collect()
    };

    let mut cache = EMOJI_CACHE.lock().await;
    for guild_id in guild_ids {
        if let Ok(emojis) = guild_id.emojis(ctx).await {
            for emoji in emojis {
                cache.insert(emoji.name.clone(), emoji);
            }
        }
    }
}

pub fn format_emoji_string(emoji: Emoji) -> String {
    return format!("<:{}:{}>", emoji.name, emoji.id);
}

async fn refresh_if_empty(ctx: &poise::serenity_prelude::Context) {
    let cache = EMOJI_CACHE.lock().await;
    let empty = cache.is_empty();
    drop(cache);
    if empty {
        refresh_emojis(ctx).await;
    }
}

pub async fn get_emojis(ctx: &poise::serenity_prelude::Context) -> HashMap<String, Emoji> {
    refresh_if_empty(ctx).await;

    let cache = EMOJI_CACHE.lock().await;
    return cache.clone();
}

pub async fn get_emoji(ctx: &poise::serenity_prelude::Context, emoji_name: &str) -> Option<Emoji> {
    refresh_if_empty(ctx).await;

    let cache = EMOJI_CACHE.lock().await;

    println!(
        "Trying to get emoji {emoji_name}, cache len {}",
        cache.len()
    );

    let result = cache.get(emoji_name);

    if let Some(emoji_name) = result {
        return Some(emoji_name.clone());
    } else {
        return None;
    }
}

pub async fn emojify_custom(ctx: Context<'_>, text: &str, emoji_pattern: &str) -> String {
    let mut new_string = "".to_string();

    for char in text.chars() {
        let char_lower = char.to_lowercase().to_string();

        // let mut string_replacement = &char_lower;

        let emoji_name = &emoji_pattern.replace("{}", &char_lower);

        println!("{emoji_name}");

        if let Some(emoji) = get_emoji(ctx.serenity_context(), emoji_name).await {
            new_string = format!("{new_string}{emoji} ")
        } else {
            new_string = new_string + &char_lower + " "
        }
    }

    return new_string.to_string();
}

pub async fn emojify_char(
    character: &char,
    emoji_pattern: Option<&str>,
    ctx: Option<Context<'_>>,
) -> Result<String, Error> {
    let char_lower = character.to_lowercase();

    if let Some(pattern) = emoji_pattern {
        let pattern_replaced = pattern.replace("{}", &char_lower.to_string());

        if let Some(context) = ctx {
            match (get_emoji(context.serenity_context(), &pattern_replaced).await) {
                Some(emoji) => return Ok(format_emoji_string(emoji)),
                None => return Ok(pattern_replaced),
            };
        }

        Ok(pattern_replaced)
    } else {
        Ok(format!(":regional_indicator_{}:", &char_lower))
    }
}

pub async fn emojify_string(message: &str) -> Result<String, Error> {
    let mut new_string = "".to_string();

    for char in message.chars() {
        new_string = new_string + &emojify_char(&char, None, None).await? + " ";
    }

    Ok(new_string.to_string())
}

pub async fn fetch_message_chain(
    ctx: &poise::serenity_prelude::Context,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
    let mut messages = Vec::new();

    // Fetch the initial message
    let message = ctx.http.get_message(channel_id, message_id).await?;
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

#[allow(dead_code)]
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

pub fn safe_to_u64(s: &str) -> u64 {
    let part_stripped = strip_non_numerical(s);

    if part_stripped.len() == 0 {
        return 0;
    }

    return part_stripped.parse::<u64>().unwrap();
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
