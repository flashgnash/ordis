use crate::common::Error as Error;
use crate::common::Context;

enum Role {
    User,
    Assistant,
    System


}

struct Message {
    role:Role,
    content:String


}

#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>) -> Result<(),Error> {

    let token = std::env::var("OPENAI_TOKEN").expect("missing OPENAI_TOKEN");
    let model = std::env::var("OPENAI_MODEL").unwrap_or("gpt-3.5-turbo".to_string());
    let authorized_user = std::env::var("OPENAI_AUTHORIZED").expect("This command will not work without the env variable OPENAI_AUTHORIZED (should contain a discord user ID)");
     
    ctx.say("Testing").await?;
    

    return Ok(());


     
}
 
