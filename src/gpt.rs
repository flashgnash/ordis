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
use std::collections::HashMap;

use crate::common::HTTP_CLIENT;


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
pub struct OpenAIFilterRequest {
    pub model: String,
    pub input: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterCategoryScores {
    pub sexual: f64,
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: f64,
    pub harassment: f64,
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: f64,
    pub hate: f64,
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: f64,
    pub illicit: f64,
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: f64,
    #[serde(rename = "self-harm")]
    pub self_harm: f64,
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: f64,
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: f64,
    pub violence: f64,
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterCategories {
    pub sexual: bool,
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: bool,
    pub harassment: bool,
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: bool,
    pub hate: bool,
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: bool,
    pub illicit: bool,
    #[serde(rename = "illicit/violent")]
    pub illicit_violent: bool,
    #[serde(rename = "self-harm")]
    pub self_harm: bool,
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: bool,
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: bool,
    pub violence: bool,
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterResult{
    pub flagged: bool,
    pub categories: FilterCategories,
    pub category_scores: FilterCategoryScores,
    pub category_applied_input_types: HashMap<String, Vec<String>>,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct FilterResponse {
    id: String,
    model: String,
    pub results: Vec<FilterResult>
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

pub async fn filter_message(message: &str) -> Result<FilterResponse,Error> {
    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    
    let client = &HTTP_CLIENT;
        let request = OpenAIFilterRequest {
        model: "omni-moderation-latest".to_string(),
        input: message.to_string()
    };

    let content = 
        client.post("https://api.openai.com/v1/moderations")
        .json(&request)
        .header(AUTHORIZATION,format!("Bearer {token}"))
        .send().await?.text().await?;

                                                                                      
    
    let response : FilterResponse = serde_json::from_str(&content)?;

    Ok(response)
}


pub async fn filter_hate(message: &str) -> Result<bool,Error> {

    let slurs_present = filter_message(message).await?.results[0].categories.hate == true;

    Ok(slurs_present)

}


#[allow(dead_code)]
pub async fn generate_to_string(model:&str,messages:Vec<Message>) -> Result<String,Error> {
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

pub async fn generate(model:&str, messages:Vec<Message>) -> Result<OpenAIResponse, Error> {                                       
    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    
    let client = &HTTP_CLIENT;
        let request = OpenAIRequest {
        model: model.to_string(),
        messages: messages 
    };

    let content = 
        client.post("https://api.openai.com/v1/chat/completions")
        .json(&request)
        .header(AUTHORIZATION,format!("Bearer {token}"))
        .send().await?.text().await?;

                                                                                      
    
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


    let input = generate("gpt-4o-mini",messages).await?;

    if let Some(result) = string_to_bool(&input.choices[0].message.content) {
        println!("Converted value: {}", result);
        return Ok(result);
    } else {
        println!("Invalid input");
        return Ok(false);
    }
    

}

pub async fn generate_ordis(message: &str) -> Result<OpenAIResponse,Error> {

    // let use_gpt_4 = model_selector(message).await?;
    let mut model = "gpt-4o-mini";

    // if use_gpt_4 {
    //     model = "gpt-4";
    // }

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
pub async fn generate_translator(message: &str, lang1: &str, lang2:&str) -> Result<OpenAIResponse,Error> {

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


    return generate("gpt-4o-mini",messages).await;

}


pub async fn translate_internal(ctx: Context<'_>,message:String) -> Result<(),Error> {
     let msg = ctx.say("*Translating, please wait...*").await?;

    let response = generate_translator(&message,"english","spanish").await?;

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

    let response = generate_ordis(&message).await?;

    let response_message = &response.choices[0].message.content;

    println!("{}",response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}
 
