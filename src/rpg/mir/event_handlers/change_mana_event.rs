use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::CreateButton;
use serde::Serialize;

use crate::common;
use crate::common::Error;
use crate::create_button_with_callback;
use crate::db;

use super::super::RpgError;

use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::EditMessage;
use serde_json::Value;

use poise::async_trait;

pub struct ChangeManaEvent;

#[derive(Serialize)]
pub struct ChangeManaEventParams {
    pub character_id: i32,
    pub mana_change: i64,
}

impl ChangeManaEvent {
    pub fn create_button(
        text: &str,
        params: &ChangeManaEventParams,
        button_style: ButtonStyle,
    ) -> Result<CreateButton, Error> {
        return create_button_with_callback::<Self, ChangeManaEventParams>(
            text,
            params,
            button_style,
        );
    }
}

#[async_trait]
impl common::EventHandlerTrait for ChangeManaEvent {
    async fn run(
        &self,
        ctx: &poise::serenity_prelude::Context,
        interaction: &poise::serenity_prelude::ComponentInteraction,
        params: &common::ButtonParams,
    ) {
        println!("Test");
        if let Some(Value::Number(char_id)) = params.get("character_id") {
            if let Some(Value::Number(mana_change)) = params.get("mana_change") {
                let channel_id = interaction.message.channel_id;
                let message_id = interaction.message.id;

                let placeholder_embed =
                    CreateEmbed::default().description("Loading... Please wait");

                channel_id
                    .edit_message(
                        &ctx,
                        message_id,
                        EditMessage::default().embed(placeholder_embed),
                    )
                    .await
                    .expect("I am so tired");

                let db_connection = &mut db::establish_connection();

                let mut char = db::characters::get(
                    char_id
                        .as_i64()
                        .ok_or(RpgError::TestingError)
                        .expect("Really gotta make these return result") as i32,
                )
                .expect("Remove this expect later");

                println!("Blah");

                char.mana = Some(
                    char.mana.unwrap_or(0)
                        + mana_change.as_i64().expect("Should always be i64") as i32,
                );

                db::characters::update(&char).expect("Argh");

                let embed = super::super::generate_status_embed(ctx, &char)
                    .await
                    .expect("Ffs");

                channel_id
                    .edit_message(&ctx, message_id, EditMessage::default().embed(embed))
                    .await
                    .expect("I am so tired");
            }
        }
    }
}
