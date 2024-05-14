use crate::common::Error as Error;
use crate::common::Context;
use reqwest;
use reqwest::header::AUTHORIZATION;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use serde;
use std::convert::From;

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
    User,
    Assistant,
    System
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

async fn generate(token: String) -> Result<(), Error> {                                       
    
    let client = &CLIENT;


    let request = OpenAI_Request {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: "Hello world".to_string(),

            }


        ]


    };
    let json = serde_json::to_string(&request)?;

    let content = 
        client.post("https://api.openai.com/v1/chat/completions")
        .body(json)
        .header(AUTHORIZATION,format!("Bearer {token}"))
        .send().await?.text().await?;

                                                                                      
    println!("{content}");                                                          
                                                                                      
    Ok(())                                                                            
}    

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>) -> Result<(),Error> {

    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    let model = std::env::var("OPENAI_MODEL").unwrap_or("gpt-3.5-turbo".to_string());
    let authorized_user = std::env::var("OPENAI_AUTHORIZED").expect("This command will not work without the env variable OPENAI_AUTHORIZED (should contain a discord user ID)");
    




    generate(token).await?;

    ctx.say("Testing").await?;
    

    return Ok(());


     
}
 
