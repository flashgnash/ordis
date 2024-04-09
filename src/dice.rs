
use rand::prelude::*;


use crate::common::Error;
use crate::common::Context;

use crate::common::safe_to_number;

use crate::common::join_to_string;
use crate::common::sum_array;



fn generate_randoms(count: i32,faces:i32) -> Vec<i32> {


       let mut rng = rand::thread_rng();

       let mut rolls : Vec<i32> = vec![];

       for _i in 0..count {
           rolls.push(rng.gen_range(1..faces));
       }

       return rolls;



}


#[poise::command(slash_command, prefix_command)]
pub async fn roll(
        ctx: Context<'_>,
        dice: String

    ) -> Result<(),Error> {

    let instances = dice.split(' ');

    let mut result: Vec<String> = vec![];

    for instance in instances {


        //Figure out what the user wants

        let mut number_of_dice = 1;
        let mut faces_of_die = 6;

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



        if number_of_dice == 0 {
            ctx.say("How am I supposed to roll 0 dice?").await?; //TODO handle errors elsewhere
            return Ok(());
        }
        if faces_of_die == 0 {
            ctx.say("How do you expect me to roll a d0?").await?;
            return Ok(());
        }


        //Roll the dice




 
        let dice_rolls = generate_randoms(number_of_dice, faces_of_die);
        //Write the messages

        let all_roles = join_to_string(&dice_rolls,",");
        let total: i32 = sum_array(dice_rolls);

        let mut message = format!("- {} D{}s: **({})**      [{}]",number_of_dice,faces_of_die,total,all_roles);
     
        if number_of_dice == 1  {
           message = format!("- A D{}: **({})**",faces_of_die, total);
        }    
        
        result.push(message);

    }


    let message = result.join("\n");
    

    ctx.say(format!("Rolling...\n{}",message)).await?;
        
    Ok(())
}


