use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::CreateButton;
use poise::serenity_prelude::CreateInteractionResponseFollowup;
use poise::serenity_prelude::CreateSelectMenuOption;
use serde::Serialize;

use crate::common;
use crate::common::Error;
use crate::create_button_with_callback;
use crate::create_select_option_with_callback;
use crate::db;
use crate::rpg::mir::get_roll_channel;

use super::super::RpgError;

use poise::serenity_prelude::CreateMessage;
use serde_json::Value;

use poise::async_trait;

pub struct RollEvent;

#[derive(Serialize)]
pub struct RollEventParams {
    pub dice_string: String,
    pub character_id: i32,
}

impl RollEvent {
    pub fn create_button(
        text: &str,
        params: &RollEventParams,
        button_style: ButtonStyle,
    ) -> Result<CreateButton, Error> {
        return create_button_with_callback::<Self, RollEventParams>(text, params, button_style);
    }

    pub fn create_select_item(
        text: &str,
        params: &RollEventParams,
    ) -> Result<CreateSelectMenuOption, Error> {
        return create_select_option_with_callback::<Self, RollEventParams>(text, params);
    }
}

#[async_trait]
impl common::EventHandlerTrait for RollEvent {
    async fn run(
        &self,
        ctx: &poise::serenity_prelude::Context,
        interaction: &poise::serenity_prelude::ComponentInteraction,
        params: &common::ButtonParams,
    ) {
        if let Some(Value::Number(char_id)) = params.get("character_id") {
            if let Some(Value::String(dice_string)) = params.get("dice_string") {
                println!("Event received with param: {}", dice_string);

                let db_connection = &mut db::establish_connection();

                let char = db::characters::get(
                    db_connection,
                    char_id
                        .as_i64()
                        .ok_or(RpgError::TestingError)
                        .expect("Really gotta make these return result") as i32,
                )
                .expect("Remove this expect later");

                let result =
                    super::super::roll_with_char_sheet(ctx, Some(dice_string.to_string()), &char)
                        .await
                        .expect("This is bad practise");

                let colour =
                    crate::common::get_user_colour(ctx, interaction.guild_id, interaction.user.id)
                        .await
                        .expect("I really have to fix this");

                let embed = crate::dice::generate_roll_embed(
                    result,
                    &char.name.unwrap_or("Test".to_string()),
                    colour,
                )
                .await
                .expect("Why did I design the event system this way");

                // ctx.send(embed).await?;

                let channel;
                if let Some(guild_id) = interaction.guild_id {
                    channel = get_roll_channel(db_connection, &guild_id).expect("woohoo");
                } else {
                    channel = None;
                }

                if let Some(channel_id) = channel {
                    channel_id
                        .send_message(ctx, CreateMessage::default().embed(embed.clone()))
                        .await
                        .expect("AAA");

                    interaction
                        .create_followup(
                            ctx,
                            CreateInteractionResponseFollowup::default()
                                .ephemeral(true)
                                .embed(embed),
                        )
                        .await
                        .expect("AAA");
                } else {
                    interaction
                        .channel_id
                        .send_message(ctx, CreateMessage::default().embed(embed))
                        .await
                        .expect("AAA");
                }
            }
        }
    }
}
