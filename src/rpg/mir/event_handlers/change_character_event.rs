use poise::serenity_prelude::ButtonStyle;
use poise::serenity_prelude::CreateButton;
use poise::serenity_prelude::CreateInteractionResponseMessage;
use poise::serenity_prelude::CreateSelectMenuOption;
use poise::serenity_prelude::CustomMessage;
use serde::Serialize;

use crate::common;
use crate::common::Error;
use crate::create_button_with_callback;
use crate::create_select_option_with_callback;
use crate::db;

use super::super::RpgError;

use poise::serenity_prelude::CreateInteractionResponseFollowup;
use poise::serenity_prelude::CreateMessage;
use serde_json::Value;

use poise::async_trait;

pub struct ChangeCharacterEvent;

#[derive(Serialize)]
pub struct ChangeCharacterEventParams {
    pub user_id: u64,
    pub character_id: i32,
}

impl ChangeCharacterEvent {
    pub fn create_select_item(
        text: &str,
        params: &ChangeCharacterEventParams,
    ) -> Result<CreateSelectMenuOption, Error> {
        return create_select_option_with_callback::<Self, ChangeCharacterEventParams>(
            text, params,
        );
    }
}

#[async_trait]
impl common::EventHandlerTrait for ChangeCharacterEvent {
    async fn run(
        &self,
        ctx: &poise::serenity_prelude::Context,
        interaction: &poise::serenity_prelude::ComponentInteraction,
        params: &common::ButtonParams,
    ) {
        println!("Hello world {:?}", params);
        if let Some(Value::Number(char_id)) = params.get("character_id") {
            if let Some(Value::Number(user_id)) = params.get("user_id") {
                let db_connection = &mut db::establish_connection();

                println!("Hello world {user_id} {char_id}");

                let waiting_message_id = interaction
                    .create_followup(
                        ctx,
                        CreateInteractionResponseFollowup::default()
                            .ephemeral(true)
                            .content("Thinking... please wait"),
                    )
                    .await
                    .expect("AAA");

                let char = db::characters::get(
                    db_connection,
                    char_id
                        .as_i64()
                        .ok_or(RpgError::TestingError)
                        .expect("Really gotta make these return result") as i32,
                )
                .expect("Remove this expect later");

                let mut user =
                    db::users::get_or_create(db_connection, user_id.as_u64().expect("ddd"))
                        .expect("asdasd");

                let comparison_user_id = interaction.user.id;

                if let Some(user_id) = char.user_id.clone() {
                    if &user_id.to_string() == &comparison_user_id.to_string() {
                        user.selected_character = Some(char_id.as_i64().expect("asdasd") as i32);
                        db::users::update(db_connection, &user).expect("I hate this system");
                        println!("aaa {char_id}");

                        let embed = super::super::generate_status_embed(ctx, &char)
                            .await
                            .expect("Ffs");

                        let char_id_i32 = char_id.as_i64().expect("Wrong char id") as i32;

                        let rows = vec![
                            // CreateActionRow::SelectMenu(select_menu),
                            super::super::advantage_roll_buttons(char_id_i32),
                            super::super::stat_roll_buttons(char_id_i32),
                            super::super::character_select_dropdown(
                                db_connection,
                                user_id.parse().unwrap(),
                            )
                            .await
                            .expect("asda"),
                        ];

                        interaction
                            .edit_followup(
                                ctx,
                                waiting_message_id,
                                CreateInteractionResponseFollowup::default()
                                    .embed(embed)
                                    .content("Switched character. Current status:")
                                    .ephemeral(true)
                                    .components(rows),
                            )
                            .await
                            .expect("AAA");
                    }
                }

                // interaction
                //     .channel_id
                //     .send_message(
                //         ctx,
                //         CreateMessage::default().content("Hello"), // CreateMessage::default().content(format!(
                //                                                    // "Rolling for {}:\n {}",
                //                                                    // interaction.user.name, result
                //                                                    // )),
                //     )
                //     .await
                //     .expect("AAA");
            }
        }
    }
}
