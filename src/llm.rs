use crate::common::Error as Error;
use crate::common::Context;
use reqwest;
use reqwest::header::AUTHORIZATION;
use serde::{Serialize, Deserialize};
use serde;
use std::convert::From;
use std::fmt;
use chrono::Utc;
use poise::CreateReply;


use crate::common::HTTP_CLIENT;
use std::collections::HashMap;

use lazy_static::lazy_static;

#[allow(dead_code)]
#[derive(Debug)]
pub enum LLMError {
    NoGuildId,

    NoProviderConfig,
    NoProviderConfigForModel


    
}

impl fmt::Display for LLMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {

            _ => write!(f,"Unknown error"),
        }
    }
}
impl std::error::Error for LLMError {}

#[derive(Debug)]
pub struct ProviderConfig {
    pub endpoint: &'static str,
    pub access_token_env: &'static str,
    pub valid_models: &'static [&'static str],
}

lazy_static! {
    pub static ref PROVIDER_CONFIGS: HashMap<&'static str, ProviderConfig> = {
        let mut map = HashMap::new();
        map.insert(
            "openai",
            ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions",
                access_token_env: "OPENAI_TOKEN",
                valid_models: &["gpt-3.5-turbo", "gpt-4", "gpt-4o-mini"],
            },
        );
        map.insert(
            "grok",
            ProviderConfig {
                endpoint: "https://api.x.ai/v1/chat/completions",
                access_token_env: "GROK_TOKEN",
                valid_models: &["grok-beta"],
            },
        );
        map
    };
}


pub fn get_provider_config(provider: &str) -> Option<&'static ProviderConfig> {
    PROVIDER_CONFIGS.get(provider)
}

pub fn get_provider_by_model(model: &str) -> Option<&'static ProviderConfig> {
    PROVIDER_CONFIGS.values().find(|config| config.valid_models.contains(&model))
}

#[derive(Serialize, Deserialize)]
pub enum Role {
    user,
    assistant,
    system
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role:Role,
    pub content:String
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model:String,
    pub messages: Vec<Message>


}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub finish_reason:String,
    pub index:i32,
    pub message:Message
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIResponse{
    pub choices:Vec<Choice>

}


#[derive(Serialize, Deserialize)]
struct DallERequest {
    pub prompt: String,
    pub n: u8,  // Number of images
    pub size: String,  // Image size e.g., "1024x1024"
}

#[derive(Serialize, Deserialize)]
struct DallEResponse {
    pub data: Vec<DallEImageData>,
}

#[derive(Serialize, Deserialize)]
struct DallEImageData {
    pub url: String,
}

impl fmt::Display for OpenAIResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.choices[0].message.content)
    }
}



#[allow(dead_code)]
pub async fn generate_to_string(model:Option<&str>,messages:Vec<Message>) -> Result<String,Error> {
    let response = generate(model,messages).await?;
    

    let response_message = &response.choices[0].message.content;
    return Ok(format!("{}",response_message));
}

pub async fn generate_image(prompt: &str) -> Result<String, Error> {
    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    
    let client = &HTTP_CLIENT;

    let request = DallERequest {
        prompt: prompt.to_string(),
        n: 1,  // Generate 1 image
        size: "1024x1024".to_string(),
    };

    let content = client
        .post("https://api.openai.com/v1/images/generations")
        .json(&request)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?
        .text()
        .await?;

    let response: DallEResponse = serde_json::from_str(&content)?;
    
    let image_data = response.data.first().expect("No image generated! (openai)"); 
    Ok(image_data.url.clone())
}




pub async fn generate(model:Option<&str>, messages:Vec<Message>) -> Result<OpenAIResponse, Error> {                                       

    let model = model.unwrap_or("gpt-4o-mini");

    let provider_config = get_provider_by_model(model).ok_or(LLMError::NoProviderConfigForModel)?;

    let token = std::env::var(provider_config.access_token_env).expect("Missing token for LLM provider");

    let endpoint = provider_config.endpoint;    
    // token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    
    let client = &HTTP_CLIENT;
        let request = OpenAIRequest {
        model: model.to_string(),
        messages: messages 
    };

    let content = 
        client.post(endpoint)
        .json(&request)
        .header(AUTHORIZATION,format!("Bearer {token}"))
        .send().await?.text().await?;

                                                                                      
    println!("{}",&content);

    let response : OpenAIResponse = serde_json::from_str(&content)?;

    Ok(response)
}    

fn string_to_bool(value: &str) -> Option<bool> {
    match value.trim().to_uppercase().as_str() {
        "Y" => Some(true),
        "N" => Some(false),
        _ => None,
    }
}


pub async fn model_selector(message:&str) -> Result<bool,Error> {
    let prompt = "
You are a programming filter.

You should respond with Y if the given request would involve generating code
You should respond with N if the given request would not involve generating code

Do not respond with anything else under any circumstances";


    let messages = vec![            

            Message {
                role: Role::system,
                content: prompt.to_string()

            },

            Message {
                role: Role::user,
                content: message.to_string(),

            }
        ];


    let input = generate(None,messages).await?;

    if let Some(result) = string_to_bool(&input.choices[0].message.content) {
        println!("Converted value: {}", result);
        return Ok(result);
    } else {
        println!("Invalid input");
        return Ok(false);
    }
    

}

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
pub async fn generate_translator(message: &str, lang1: &str, lang2:&str,model: Option<&str>) -> Result<OpenAIResponse,Error> {

    let now = Utc::now();
    let messages = vec![            
            Message {
                role: Role::system,
                content: format!("The current time is {}",now.format("%Y-%m-%d %H:%M:%S"))

            },


            Message {
                role: Role::system,
                content: "You are Ordis, the helpful AI assistant from the game Warframe. You should take on Ordis's personality when responding to prompts, while still being helpful and accurate".to_string()

            },
            Message {
                role: Role::system,
                content: format!("Act as a {lang1}-{lang2} translator. Respond with only an accurate translation and nothing else. Please translate to natural speech in the given language")

            },

            Message {
                role: Role::user,
                content:message.to_string(),

            }
        ];


    return generate(model,messages).await;

}


pub async fn translate_internal(ctx: Context<'_>,message:String) -> Result<(),Error> {
     let msg = ctx.say("*Translating, please wait...*").await?;

    let response = generate_translator(&message,"english","spanish",None).await?;

    let response_message = &response.choices[0].message.content;

    println!("{}",response_message);

    let message_text = format!("translation of: ``{message}``\n\n{response_message}");

    let reply = CreateReply::default().content(message_text);
    
    msg.edit(ctx, reply).await?;

    Ok(())   
}



#[poise::command(context_menu_command = "Translate message")]
pub async fn translate_context(
    ctx: Context<'_>,
     #[description = "Message to translate"] msg: crate::serenity::Message
     ) -> Result<(),Error> {

      translate_internal(ctx,msg.content).await?;

      Ok(())
}



#[poise::command(slash_command, prefix_command)]
pub async fn translate(ctx: Context<'_>, message:String) -> Result<(),Error> {

    translate_internal(ctx,message).await?;

    return Ok(());
}


 
#[poise::command(slash_command, prefix_command)]
pub async fn draw(ctx: Context<'_>, message:String) -> Result<(),Error> {

    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response_message = generate_image(&message).await?;


    println!("Generated image URL: {}",response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}
 

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>, message:String) -> Result<(),Error> {

    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response = generate_ordis(&message,None).await?;

    let response_message = &response.choices[0].message.content;

    println!("{}",response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}
 