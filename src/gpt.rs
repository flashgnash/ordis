use crate::common::Error as Error;
use crate::common::Context;
use reqwest;
use reqwest::header::AUTHORIZATION;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use serde;
use std::convert::From;
use std::fmt;

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

async fn generate(token: String, message:&str) -> Result<OpenAI_Response, Error> {                                       
    
    let client = &CLIENT;


    let request = OpenAI_Request {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            Message {
                role: Role::system,
                content: "You are Ordis, the helpful AI assistant from the game Warframe. You should take on Ordis's personality when responding to prompts, while still being helpful and accurate".to_string()

            },
            Message {
                role: Role::user,
                content:message.to_string(),

            }


        ]


    };


    let content = 
        client.post("https://api.openai.com/v1/chat/completions")
        .json(&request)
        .header(AUTHORIZATION,format!("Bearer {token}"))
        .send().await?.text().await?;

                                                                                      
    println!("{content}");                                                          
    
    let response : OpenAI_Response = serde_json::from_str(&content).unwrap();

    Ok(response)
}    

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>, message:String) -> Result<(),Error> {

    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    let model = std::env::var("OPENAI_MODEL").unwrap_or("gpt-3.5-turbo".to_string());
    let authorized_user = std::env::var("OPENAI_AUTHORIZED").expect("This command will not work without the env variable OPENAI_AUTHORIZED (should contain a discord user ID)");
    




    let response = generate(token,&message).await?;

    let response_message = &response.choices[0].message.content;

    ctx.say(response_message).await?;
    

    return Ok(());


     
}
 
