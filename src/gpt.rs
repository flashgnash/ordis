use crate::common::Error as Error;
use crate::common::Context;
use reqwest;
use reqwest::header::AUTHORIZATION;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use serde;
use std::convert::From;
use std::fmt;
use chrono::Utc;

lazy_static! {                                                                                                                                                                   
    static ref CLIENT: reqwest::Client = reqwest::Client::new();                                                                                                                                   
}  


// enum GptError {                                                                                                                                                                   
//     Reqwest(reqwest::Error),                                                                                                                                                     
//     Json(serde_json::Error),                                                                                                                                                     
// }        
//

// impl From<reqwest::Error> for GptError {
//     fn from(err: reqwest::Error) -> GptError {
//         GptError::Reqwest(err)
//     }
// }                                                                                                                                                                                
//                                                                                                                                                                                  
// impl From<serde_json::Error> for GptError {
//     fn from(err: serde_json::Error) -> GptError {
//         GptError::Json(err)
//     }
// }     
//
//    impl fmt::Display for GPTError {
//         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             match *self {
//                 GPTError::Reqwest(ref err) => err.fmt(f),
//                 GPTError::Json(ref err) => err.fmt(f),
//             }                                                                                                                                                                        
//         }                                                                                                                                                                            
//     }                                                                                                                                                                                
// impl fmt::Debug for GPTError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             GPTError::Reqwest(ref err) => err.fmt(f),
//             GPTError::Json(ref err) => err.fmt(f),
//         }
//     }
// }
//
// impl std::error::Error for GPTError {}     
//
//
//
//
#[derive(Serialize, Deserialize)]
enum Role {
    user,
    assistant,
    system
}

#[derive(Serialize, Deserialize)]
struct Message {
    role:Role,
    content:String
}

#[derive(Serialize, Deserialize)]
struct OpenAI_Request {
    model:String,
    messages: Vec<Message>


}

#[derive(Serialize, Deserialize)]
struct Choice {
    finish_reason:String,
    index:i32,
    message:Message
}

#[derive(Serialize, Deserialize)]
struct OpenAI_Response {
    choices:Vec<Choice>

}

impl fmt::Display for OpenAI_Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.choices[0].message.content)
    }
}



async fn generate(token: &String,model:&str, messages:Vec<Message>) -> Result<OpenAI_Response, Error> {                                       
    
    let client = &CLIENT;
    

    let request = OpenAI_Request {
        model: model.to_string(),
        messages: messages 


    };


    let content = 
        client.post("https://api.openai.com/v1/chat/completions")
        .json(&request)
        .header(AUTHORIZATION,format!("Bearer {token}"))
        .send().await?.text().await?;

                                                                                      
    //println!("{content}");                                                          
    
    let response : OpenAI_Response = serde_json::from_str(&content)?;

    Ok(response)
}    

fn string_to_bool(value: &str) -> Option<bool> {
    match value.trim().to_uppercase().as_str() {
        "Y" => Some(true),
        "N" => Some(false),
        _ => None,
    }
}


pub async fn model_selector(token:&String, message:&str) -> Result<bool,Error> {
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
                content:message.to_string(),

            }
        ];


    let input = generate(&token,"gpt-4o-mini",messages).await?;

    if let Some(result) = string_to_bool(&input.choices[0].message.content) {
        println!("Converted value: {}", result);
        return Ok(result);
    } else {
        println!("Invalid input");
        return Ok(false);
    }
    

}


pub async fn generate_ordis(token:&String, message: &str) -> Result<OpenAI_Response,Error> {


    let use_gpt_4 = model_selector(token,message).await?;
    let mut model = "gpt-3.5-turbo";

    if use_gpt_4 {
        model = "gpt-4o";
    }

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


    return generate(token,model,messages).await;

}
pub async fn generate_translator(token:String, message: &str, lang1: &str, lang2:&str) -> Result<OpenAI_Response,Error> {

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


    return generate(&token,"gpt-3.5-turbo",messages).await;

}



#[poise::command(slash_command, prefix_command)]
pub async fn translate(ctx: Context<'_>, message:String) -> Result<(),Error> {

    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    let model = std::env::var("OPENAI_MODEL").unwrap_or("gpt-3.5-turbo".to_string());
    let authorized_user = std::env::var("OPENAI_AUTHORIZED").expect("This command will not work without the env variable OPENAI_AUTHORIZED (should contain a discord user ID)");
    

    let msg = ctx.say("*Translating, please wait...*").await?;

    let response = generate_translator(token,&message,"english","spanish").await?;

    let response_message = &response.choices[0].message.content;

    println!("{}",response_message);

    msg.edit(ctx, |m| m.content(format!("Translation of: ``{message}``\n\n{response_message}"))).await?;

    return Ok(());
}
 

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>, message:String) -> Result<(),Error> {

    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    let model = std::env::var("OPENAI_MODEL").unwrap_or("gpt-3.5-turbo".to_string());
    let authorized_user = std::env::var("OPENAI_AUTHORIZED").expect("This command will not work without the env variable OPENAI_AUTHORIZED (should contain a discord user ID)");
   


    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response = generate_ordis(&token,&message).await?;

    let response_message = &response.choices[0].message.content;

    println!("{}",response_message);

    msg.edit(ctx, |m| m.content(response_message)).await?;

    return Ok(());
}
 
