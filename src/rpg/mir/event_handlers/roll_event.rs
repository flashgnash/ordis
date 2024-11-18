use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::CreateButton;
use serde::Serialize;

use crate::common;
use crate::common::Error;
use crate::create_button_with_callback;
use crate::db;

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

                let (result, _) =
                    super::super::roll_with_char_sheet(ctx, Some(dice_string.to_string()), char)
                        .await
                        .expect("This is bad practise");

                interaction
                    .channel_id
                    .send_message(
                        ctx,
                        CreateMessage::default().content(format!(
                            "Event received with param: {} from user {},{}",
                            dice_string, interaction.user.name, result
                        )),
                    )
                    .await
                    .expect("AAA");
            }
        }
    }
}
