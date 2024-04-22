use crate::common::Error as Error;
use crate::common::Context;
use reqwest;

enum Role {
    User,
    Assistant,
    System


}

struct Message {
    role:Role,
    content:String


}

async fn auth() -> Result<(), reqwest::Error> {                                       
    let content = reqwest::get("https://www.rust-lang.org").await?
        .text() .await?;
                                                                                      
    println!("{}", content);                                                          
                                                                                      
    Ok(())                                                                            
}    



async fn generate(_token: String) -> Result<(), reqwest::Error> {                                       
    let content = reqwest::get("https://api.openai.com/v1/chat/completions")
//        .header(AUTHORIZATION, format!("Bearer {}",token))
        .await?
        .text().await?;
                                                                                      
    println!("{}", content);                                                          
                                                                                      
    Ok(())                                                                            
}    

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>) -> Result<(),Error> {

    let _token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    let _model = std::env::var("OPENAI_MODEL").unwrap_or("gpt-3.5-turbo".to_string());
    let _authorized_user = std::env::var("OPENAI_AUTHORIZED").expect("This command will not work without the env variable OPENAI_AUTHORIZED (should contain a discord user ID)");
    

    auth().await?;

    ctx.say("Testing").await?;
    

    return Ok(());


     
}
 
