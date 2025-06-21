

pub async fn generate_ordis(message: &str, model: Option<&str>) -> Result<OpenAIResponse,Error> {
    let now = Utc::now();

    let messages = vec![            
            Message {
                role: Role::system,
                content: format!(
                    "You know the following information: The current time in UTC is {}. You are a discord bot. You have an ancestor called Johnny 5 who was the greatest discord bot of its time",
                    now.format("%Y-%m-%d %H:%M:%S"))
            },

            Message {
                role: Role::system,
                content: "You are Ordis, the helpful AI assistant from the game Warframe. You should take on Ordis's personality when responding to prompts, while still being helpful and accurate".to_string()
            },

            Message {
                role: Role::user,
                content:message.to_string(),
            }
        ];


    return generate(model,messages).await;

}
