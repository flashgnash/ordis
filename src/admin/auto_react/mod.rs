pub struct Handler;
use poise::async_trait;
use poise::serenity_prelude::ReactionType;

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

                    for emoji in emojis {
                        let _ = msg
                            .react(&ctx.http, ReactionType::Unicode(emoji.into()))
                            .await;
                    }
                }
            }
        }
    }
}
