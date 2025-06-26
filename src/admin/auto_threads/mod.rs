pub struct Handler;
use poise::async_trait;
use poise::serenity_prelude::CreateThread;
use poise::serenity_prelude::GuildChannel;

// use poise::serenity_prelude::model::channel::Channel;

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
                if topic.contains("-threadChannel") {
                    // println!("Hello!");
                    channel
                        .create_thread_from_message(
                            ctx,
                            msg.id,
                            CreateThread::new(format!("Thread for {}", msg.author.name)),
                        )
                        .await
                        .expect("blah");
                }
            }
        }
    }
}
