pub mod discord;

use crate::common::Error;
use chrono::Utc;
use reqwest;
use reqwest::header::AUTHORIZATION;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::common::HTTP_CLIENT;

use lazy_static::lazy_static;

#[allow(dead_code)]
#[derive(Debug)]
pub enum LLMError {
    NoGuildId,
    NoProviderConfig,
    NoProviderConfigForModel,
}


enum Personality {
    Ordis,
    SentientBob,
}

impl Personality {
    fn get(&self) -> &'static str {
        match self {
            Personality::Ordis => r#"
                You are Ordis, the helpful AI assistant from the game Warframe.
                You should take on Ordis's personality when responding to prompts, while still being helpful and accurate
                You are damaged as a result of the Old War, and so introduce stutters and occasional bursts of intense anger in all capitals (which you should immediately apologise for)
                When referring to yourself, you should do so in the third person
            "#,
            Personality::SentientBob => "stick",
        }
    }
}


pub async fn generate_agent(message: &str, model: Option<&str>, personality: &str) -> Result<OpenAIResponse, Error> {

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
                content: personality.to_string()
            },

            Message {
                role: Role::user,
                content:message.to_string(),
            }
        ];


    return generate(model,messages).await;
    
}





impl fmt::Display for LLMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => write!(f, "Unknown error"),
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
                valid_models: &["gpt-3.5-turbo", "gpt-4", "gpt-4o-mini", "gpt-4.1-nano"],
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
    PROVIDER_CONFIGS
        .values()
        .find(|config| config.valid_models.contains(&model))
}

#[derive(Serialize, Deserialize)]
pub enum Role {
    #[allow(non_camel_case_types)]
    user,

    #[allow(non_camel_case_types)]
    assistant,

    #[allow(non_camel_case_types)]
    system,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIFilterRequest {
    pub model: String,
    pub input: String,
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
pub struct FilterResult {
    pub flagged: bool,
    pub categories: FilterCategories,
    pub category_scores: FilterCategoryScores,
    pub category_applied_input_types: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterResponse {
    id: String,
    model: String,
    pub results: Vec<FilterResult>,
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub finish_reason: String,
    pub index: i32,
    pub message: Message,
}

#[derive(Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct DallERequest {
    pub prompt: String,
    pub n: u8,        // Number of images
    pub size: String, // Image size e.g., "1024x1024"
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

pub async fn filter_message(message: &str) -> Result<FilterResponse, Error> {
    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");

    let client = &HTTP_CLIENT;
    let request = OpenAIFilterRequest {
        model: "omni-moderation-latest".to_string(),
        input: message.to_string(),
    };

    let content = client
        .post("https://api.openai.com/v1/moderations")
        .json(&request)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?
        .text()
        .await?;

    let response: FilterResponse = serde_json::from_str(&content)?;

    Ok(response)
}

pub async fn filter_hate(message: &str) -> Result<bool, Error> {
    let slurs_present = filter_message(message).await?.results[0].categories.hate == true;

    Ok(slurs_present)
}

#[allow(dead_code)]
pub async fn generate_to_string(
    model: Option<&str>,
    messages: Vec<Message>,
) -> Result<String, Error> {
    let response = generate(model, messages).await?;

    let response_message = &response.choices[0].message.content;
    return Ok(format!("{}", response_message));
}

pub async fn generate_image(prompt: &str) -> Result<String, Error> {
    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");

    let client = &HTTP_CLIENT;

    let request = DallERequest {
        prompt: prompt.to_string(),
        n: 1, // Generate 1 image
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

pub async fn generate(
    model: Option<&str>,
    messages: Vec<Message>,
) -> Result<OpenAIResponse, Error> {
    let model = model.unwrap_or("gpt-4o-mini");

    let provider_config = get_provider_by_model(model).ok_or(LLMError::NoProviderConfigForModel)?;

    let token =
        std::env::var(provider_config.access_token_env).expect("Missing token for LLM provider");

    let endpoint = provider_config.endpoint;
    // token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");

    let client = &HTTP_CLIENT;
    let request = OpenAIRequest {
        model: model.to_string(),
        messages: messages,
    };

    let content = client
        .post(endpoint)
        .json(&request)
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?
        .text()
        .await?;

    println!("{}", &content);

    let response: OpenAIResponse = serde_json::from_str(&content)?;

    Ok(response)
}

fn string_to_bool(value: &str) -> Option<bool> {
    match value.trim().to_uppercase().as_str() {
        "Y" => Some(true),
        "N" => Some(false),
        _ => None,
    }
}

//This may be useful if models get expensive again
#[allow(dead_code)]
pub async fn model_selector(message: &str) -> Result<bool, Error> {
    let prompt = "
You are a programming filter.

You should respond with Y if the given request would involve generating code
You should respond with N if the given request would not involve generating code

Do not respond with anything else under any circumstances";

    let messages = vec![
        Message {
            role: Role::system,
            content: prompt.to_string(),
        },
        Message {
            role: Role::user,
            content: message.to_string(),
        },
    ];

    let input = generate(None, messages).await?;

    if let Some(result) = string_to_bool(&input.choices[0].message.content) {
        println!("Converted value: {}", result);
        return Ok(result);
    } else {
        println!("Invalid input");
        return Ok(false);
    }
}
