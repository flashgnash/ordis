use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}


fn strip_non_numerical(s: &str) -> String {                                                   
    s.chars().filter(|c| c.is_digit(10)).collect()                                    
}    

fn safe_to_number(s: &str) -> i32 {

       let part_stripped = strip_non_numerical(s);

        if part_stripped.len() == 0 {
            return 0;
        }

       return part_stripped.parse::<i32>().unwrap();

}

#[poise::command(slash_command, prefix_command)]
async fn roll(
        ctx: Context<'_>,
        dice: String

    ) -> Result<(),Error> {

    let instances = dice.split(' ');

    let mut result: Vec<&str> = vec![0];

    let mut number_of_dice = 1;
    let mut faces_of_die = 6;

    for instance in instances {
﻿

        let components : Vec<&str> = instance.split('d').collect();
        

        if components[0] == "" {
            faces_of_die = safe_to_number(components[1]); 
        }
        else if components.len() == 2 {
            faces_of_die = safe_to_number(components[1]); 
            number_of_dice = safe_to_number(components[0]); 
        }
        else {
            ctx.say("Too much D (you had more than one of the letter d in one of your rolls)").await?;
        }

        let instance_number = safe_to_number(instance);


        if(number_of_dice == 0){
            ctx.say("How am I supposed to roll 0 dice?").await?; //TODO handle errors elsewhere
            return Ok(());
        }
        if(faces_of_die == 0){
            ctx.say("How do you expect me to roll a d0?").await?;
            return Ok(());
        }

        let message = format!("{} d{}s",number_of_dice,faces_of_die);
     
        if(number_of_dice == 1) {
           message = format!("a d{}",faces_of_die);
        }    
        
        result.push(message);

    }


    let message = result.join("\n");
    

    ctx.say(format!("Rolling... {}",message)).await?;
        
    Ok(())
}



#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(),Error> {

    ctx.say("Testing").await?;
    Ok(())
    
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(),ping(),roll()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
