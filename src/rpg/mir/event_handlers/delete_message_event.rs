use crate::common::EventHandlerTrait;
use async_trait::async_trait;
use poise::serenity_prelude::{ButtonStyle, ComponentInteraction, Context, CreateButton};
use serde::{Deserialize, Serialize};

pub struct DeleteMessageEvent;

#[derive(Serialize, Deserialize)]
pub struct DeleteMessageEventParams {}

#[async_trait]
impl EventHandlerTrait for DeleteMessageEvent {
    async fn run(
        &self,
        ctx: &Context,
        interaction: &ComponentInteraction,
        _params: &crate::common::ButtonParams,
    ) {
        if let Err(e) = interaction.message.delete(&ctx.http).await {
            println!("Failed to delete message: {:?}", e);
        }
    }
}

impl DeleteMessageEvent {
    pub fn create_button(
        text: &str,
        _params: &DeleteMessageEventParams,
    ) -> Result<CreateButton, Box<dyn std::error::Error + Send + Sync>> {
        let callback_name = std::any::type_name::<DeleteMessageEvent>()
            .split("::")
            .last()
            .unwrap()
            .to_string();

        let callback_serializable = crate::Callback {
            name: callback_name,
            params: _params,
        };

        let json = serde_json::to_string(&callback_serializable)?;

        Ok(CreateButton::new(json).label(text).style(ButtonStyle::Danger))
    }
}