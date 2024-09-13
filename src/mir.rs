use crate::common::safe_to_number;
use crate::common::Context;
use crate::common::Error;
use crate::db;
use crate::db::models::Character;

use crate::dice;
use crate::stat_puller;
use crate::stat_puller::StatPullerError;

use poise::CreateReply;
use serde_json::Value;

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
pub async fn roll(ctx: Context<'_>, dice_expression: Option<String>) -> Result<(), Error> {
    let dice = dice_expression.unwrap_or("1d100".to_string());
    let stat_block_result = stat_puller::get_stat_block_json(&ctx).await;

    let mut str_replaced = dice;

    let mut nag_user_about_character_sheet = false;

    match stat_block_result {
        Ok((response_message, _)) => {
            let stat_block: Value = serde_json::from_str(&response_message)?;

            if let Some(stats) = stat_block.get("stats") {
                if let Some(stats_object) = stats.as_object() {
                    for (stat, value) in stats_object {
                        println!("{stat}: {value}");
                        if let Some(int_value) = value.as_i64() {
                            let stat_mod = int_value / 10;
                            str_replaced = str_replaced.replace(stat, &stat_mod.to_string());
                        }
                    }
                }
            }
        }

        Err(e) => {
            if let Some(stat_puller_error) = e.downcast_ref::<StatPullerError>() {
                match stat_puller_error {
                    StatPullerError::NoCharacterSheet => {
                        // Handle specific error
                        println!("Caught NoCharacterSheet error");

                        nag_user_about_character_sheet = true;
                    }
                    _ => return Err(e), // Propagate other StatPullerError variants
                }
            } else {
                // Propagate other errors that are not StatPullerError
                return Err(e);
            }
        }
    }

    let results = dice::roll_internal(&str_replaced).await?;

    dice::output_roll_messages(ctx, results).await?;

    if nag_user_about_character_sheet {
        let character_sheet_missing_message = CreateReply::default()
                            .content("Hint: if you configure a character sheet you can add stat modifiers to your rolls (e.g /roll 1d100+str )")
                            .ephemeral(true);

        let _ = ctx.send(character_sheet_missing_message).await?;
    }

    Ok(())
}

#[poise::command(context_menu_command = "Set as character sheet")]
pub async fn setup_character_sheet(
    ctx: Context<'_>,
    #[description = "Message to use as character sheet"] msg: crate::serenity::Message,
) -> Result<(), Error> {
    let db_connection = &mut db::establish_connection();

    let author = &ctx.author();
    let user_id = author.id.get();

    let mut user = db::users::get_or_create(db_connection, user_id)?;

    let message_id = msg.id;
    let channel_id = msg.channel_id;

    let message_content = msg.content;

    user.stat_block_message_id = Some(message_id.to_string());
    user.stat_block_channel_id = Some(channel_id.to_string());

    let saved_message_id = &user
        .stat_block_message_id
        .clone()
        .expect("Somehow saved_message_id was null");
    let saved_channel_id = &user
        .stat_block_channel_id
        .clone()
        .expect("Somehow saved_channel_id was null");

    let _ = db::users::update(db_connection, &user);

    let reply = CreateReply::default()
        .content(format!(
            "Saved your character sheet as {saved_message_id} in channel {saved_channel_id}\n```{message_content}```"
        ))
        .ephemeral(true);

    let _ = ctx.send(reply).await;

    Ok(())
}

#[poise::command(context_menu_command = "Create character")]
pub async fn create_character(
    ctx: Context<'_>,
    msg: crate::serenity::Message,
) -> Result<(), Error> {
    let db_connection = &mut db::establish_connection();

    let author = &ctx.author();

    let user_id = author.id.get();
    let user_name = &author.name;

    let new_character = Character {
        name: Some("Hank".to_string()),
        id: user_id.to_string() + "_" + "Hank",
        user_id: user_id.to_string(),

        stat_block: None,
        stat_block_hash: None,

        stat_block_message_id: None,
        stat_block_channel_id: None,
    };

    let _ = db::characters::create(db_connection, &new_character)?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn get_characters(ctx: Context<'_>) -> Result<(), Error> {
    let db_connection = &mut db::establish_connection();

    let author = &ctx.author();
    let user_id = author.id.get();

    let characters = db::characters::get_from_user_id(db_connection, user_id)?;

    let num_characters = characters.len();
    let mut character_messages: Vec<String> = vec![];

    for character in characters {
        let character_name = character.name.unwrap_or("No name provided".to_string());
        let character_id = character.id;

        character_messages.push(format!("- {character_id}: {character_name}"))
    }

    let character_list_message = "Characters:\n".to_string() + &character_messages.join("\n");

    let reply = CreateReply::default()
        .content(format!(
            "You have ({num_characters}) character(s): {character_list_message}"
        ))
        .ephemeral(true);

    let _ = ctx.send(reply).await;

    Ok(())
}
#[poise::command(slash_command, prefix_command)]
pub async fn level_up(ctx: Context<'_>, num_levels: i32) -> Result<(), Error> {
    let (response_message, stat_message_raw) = stat_puller::get_stat_block_json(&ctx).await?;

    let stats: Value = serde_json::from_str(&response_message)?;

    let reply = CreateReply::default()
        .content(format!(
            "Original stat block text:\n```{}```",
            stat_message_raw
        ))
        .ephemeral(true);

    let _ = ctx.send(reply).await;

    let msg = ctx.say("*Thinking, please wait...*").await?;

    println!("{}", stats);

    let hit_die = stats.get("hit_die_per_level").unwrap().to_string();
    let stat_die = stats.get("stat_die_per_level").unwrap().to_string();
    let spell_die = stats.get("spell_die_per_level").unwrap().to_string();

    let mut hit_die_sum: i32 = 0;
    let mut stat_die_sum: i32 = 0;
    let mut spell_die_sum: i32 = 0;

    let mut message = format!(
        "Per Level: \nHit: {hit_die} \\| Stat: {stat_die} \\| Spell: {spell_die}\n------------------------------------\nRolls:"
    );

    for i in 1..num_levels + 1 {
        let (hit_die_result, _) = dice::roll_replace(&hit_die.as_str())?;
        let (stat_die_result, _) = dice::roll_replace(&stat_die.as_str())?;
        let (spell_die_result, _) = dice::roll_replace(&spell_die.as_str())?;

        hit_die_sum = hit_die_sum + safe_to_number(&hit_die_result);
        stat_die_sum = stat_die_sum + safe_to_number(&stat_die_result);
        spell_die_sum = spell_die_sum + safe_to_number(&spell_die_result);

        message = format!(
            "{message}\n\n{i})       :heart: {hit_die_result}         :hash: {stat_die_result}         :magic_wand: {spell_die_result}"
        );
    }
    message = message.replace('"', "");
    message = format!("{message}\n\n**Total**:\n           :heart: {hit_die_sum}         :hash: {stat_die_sum}         :magic_wand: {spell_die_sum}");
    let reply = CreateReply::default().content(message);
    msg.edit(ctx, reply).await?;

    return Ok(());
}
