pub struct Handler;
use poise::async_trait;
use poise::serenity_prelude::ReactionType;

use crate::common::{self, get_emoji};

use emojis;

// use poise::serenity_prelude::model::channel::Channel;

static REACT_FLAG: &str = "-autoReact";

#[async_trait]
impl poise::serenity_prelude::EventHandler for Handler {
    async fn message(
        &self,
        ctx: poise::serenity_prelude::Context,
        msg: poise::serenity_prelude::Message,
    ) {
        if msg.author.bot {
            return;
        }

        // let channel = msg.channel(&ctx).await.expect("Blah");
        let guild_channel = msg.channel_id.to_channel(&ctx).await.expect("Blah").guild();

        if let Some(channel) = &guild_channel {
            if let Some(topic) = &channel.topic {
                if let Some(start) = topic.find("-autoReact ") {
                    let rest = &topic[start + "-autoReact ".len()..];
                    let emojis = rest
                        .lines()
                        .next()
                        .unwrap_or("")
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty());

                    for emoji_string in emojis {
                        let emoji = get_emoji(&ctx, emoji_string).await;
                        println!("Ooga");

                        let reaction: ReactionType;

                        if let Some(emoji) = emoji {
                            println!("Booga {}", emoji);

                            reaction = ReactionType::Custom {
                                id: emoji.id,
                                animated: false,
                                name: Some("SOme emoji name".to_string()),
                            };
                        } else {
                            let emoji_unicode = emojis::get_by_shortcode(
                                &emoji_string.replace(":", "").to_string(),
                            );
                            if let Some(emoji_unicode_char) = emoji_unicode {
                                reaction =
                                    ReactionType::Unicode(emoji_unicode_char.as_str().to_string());
                            } else {
                                reaction = ReactionType::Unicode("❓".to_string());
                            }
                        }

                        let _ = msg.react(&ctx.http, reaction).await;
                    }
                }
            }
        }
    }
}
