use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::CreateButton;
use serde::Serialize;

use crate::common;
use crate::common::Error;
use crate::create_button_with_callback;
use crate::db;

use super::super::RpgError;

use poise::serenity_prelude::EditMessage;
use serde_json::Value;

use poise::async_trait;

pub struct UpdateStatusEvent;

#[derive(Serialize)]
pub struct UpdateStatusEventParams {
    pub character_id: i32,
}

impl UpdateStatusEvent {
    pub fn create_button(
        text: &str,
        params: &UpdateStatusEventParams,
    ) -> Result<CreateButton, Error> {
        return create_button_with_callback::<Self, UpdateStatusEventParams>(
            text,
            params,
            ButtonStyle::Primary,
        );
    }
}

#[async_trait]
impl common::EventHandlerTrait for UpdateStatusEvent {
    async fn run(
        &self,
        ctx: &poise::serenity_prelude::Context,
        interaction: &poise::serenity_prelude::ComponentInteraction,
        params: &common::ButtonParams,
    ) {
        if let Some(Value::Number(char_id)) = params.get("character_id") {
            let db_connection = &mut db::establish_connection();

            let char = db::characters::get(
                db_connection,
                char_id
                    .as_i64()
                    .ok_or(RpgError::TestingError)
                    .expect("Really gotta make these return result") as i32,
            )
            .expect("Remove this expect later");

            let channel_id = interaction.message.channel_id;
            let message_id = interaction.message.id;

            println!("Channel: {channel_id}, message: {message_id}");

            // let msg = common::fetch_message(ctx, channel_id, message_id)
            //     .await
            //     .expect("Blah");

            // println!("{}", msg.content);

            println!("{}", char_id);

            //TODO this is horrendous
            let stat_block: super::super::StatBlock = super::super::super::get_sheet(ctx, &char)
                .await
                .expect("blah");

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
