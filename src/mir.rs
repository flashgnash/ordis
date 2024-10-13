use crate::common;
use crate::common::safe_to_number;
use crate::common::Context;
use crate::common::Error;
use crate::db;
use crate::db::models::Character;

use crate::dice;
use crate::stat_puller;
use crate::stat_puller::StatPullerError;

use diesel::expression::AsExpression;
use poise::CreateReply;
use serde_json::Value;

use regex::Regex;

#[poise::command(slash_command, prefix_command)]
pub async fn get_spell(ctx: Context<'_>, spell_name: String) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    _ = ctx.send(placeholder).await?;

    let (stat_block_result, _) = stat_puller::get_stat_block_json(&ctx).await?;

    let stat_block: Value = serde_json::from_str(&stat_block_result)?;

    let (spell_block_result, _) = stat_puller::get_spell_block_json(&ctx).await?;

    let spell_block: Value = serde_json::from_str(&spell_block_result)?;

    if let Some(energy_pool) = stat_block.get("energy_pool") {
        ctx.reply(format!("Maximum Energy: {energy_pool}")).await?;
    }

    if let Some(spell_list) = spell_block.get("spells") {
        if let Some(spell) = spell_list.get(&spell_name) {
            let mut spell_cost_word = "Cost";
            let mut spell_cost_string = "unknown".to_string();

            if let Some(spell_cost) = spell.get("cost") {
                if let Some(spell_cost_i64) = spell_cost.as_i64() {
                    spell_cost_string = spell_cost_i64.to_string();

                    if spell_cost_i64 > 0 {
                        spell_cost_word = "Gain";
                    }
                }
            }

            let spell_type = spell
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("{unknown}");

            let cast_time = spell
                .get("cast_time")
                .and_then(|v| v.as_str())
                .unwrap_or("No cast time found");

            let spell_name = common::capitalize_first_letter(&spell_name);

            let msg = format!(
                "**{spell_name}**:\n> Type: {spell_type}\n> Energy {spell_cost_word}: {spell_cost_string}\n> Cast time: {cast_time}\n",
            );

            ctx.reply(msg).await?;
        } else {
            let mut spell_list_message = "Spell not found. Available Spells: \n".to_string();

            for (spell_name, _) in spell_list.as_object().expect("No spell list") {
                spell_list_message += &format!("- {spell_name} \n");
            }

            let spell_list_reply = CreateReply::default()
                .content(spell_list_message)
                .ephemeral(true);

            ctx.send(spell_list_reply).await?;
            // return Err(Box::new(crate::stat_puller::StatPullerError::SpellNotFound));
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice_expression: Option<String>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    _ = ctx.send(placeholder).await?;

    let stat_block_result = stat_puller::get_stat_block_json(&ctx).await;

    //TODO make this default configurable per server
    let mut dice = dice_expression.unwrap_or("1d100".to_string());

    //Replace d100 with 1d100, d6 with 1d6 etc

    let re = Regex::new(r"(^|[^\d])d(\d+)").unwrap();

    dice = re.replace_all(&dice, "1d$11d$2").to_string();

    println!("{}", dice);

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

#[poise::command(context_menu_command = "Create character")]
pub async fn create_character(
    ctx: Context<'_>,
    msg: crate::serenity::Message,
) -> Result<(), Error> {
    let placeholder_message = ctx
        .send(
            CreateReply::default()
                .content("Thinking... please wait")
                .ephemeral(true),
        )
        .await?;
    let db_connection = &mut db::establish_connection();

    let author = &ctx.author();

    let user_id = author.id.get();

    let result =
        db::characters::get_by_char_sheet(db_connection, msg.channel_id.get(), msg.id.get());

    match result {
        Ok(character) => {
            let character_name = character.name.unwrap_or("No Name".to_string());
            let character_id = character.id.unwrap_or(-1);

            placeholder_message
                .edit(
                    ctx,
                    CreateReply::default()
                        .content(format!("There is already a character using that character sheet! (id {character_id}, name {character_name})"))
                        .ephemeral(true),
                )
                .await?;

            return Ok(());
        }
        Err(e) => {
            println!("No existing char found using that character sheet");
        }
    }

    let response_message =
        stat_puller::get_stat_block_json_from_message(&ctx, msg.channel_id, msg.id).await?;

    println!("{}", response_message);

    let stats: Value = serde_json::from_str(&response_message)?;

    // let character_name = stats.get("name");

    if let Some(character_name) = stats.get("name") {
        if character_name.to_string() == "null" {
            let reply = CreateReply::default()
                .content(format!("Error: No character name found"))
                .ephemeral(true);

            let _ = placeholder_message.edit(ctx, reply).await;

            return Ok(());
        }
        println!("{}", stats.get("name").unwrap());

        let character_name_stringified = character_name
            .as_str()
            .expect("Character name was not a string?!")
            .to_string();

        let new_character = Character {
            name: Some(character_name_stringified.clone()),
            id: None,
            user_id: Some(user_id.to_string()),

            stat_block: None,
            stat_block_hash: None,

            stat_block_message_id: Some(msg.id.to_string()),
            stat_block_channel_id: Some(msg.channel_id.to_string()),

            spell_block: None,
            spell_block_hash: None,
            spell_block_message_id: None,
            spell_block_channel_id: None,
        };

        let _ = db::characters::create(db_connection, &new_character)?;

        let new_character = db::characters::get_latest(db_connection, user_id)?;

        let mut user = db::users::get_or_create(db_connection, user_id)?;

        let mut extra_text: &str = "";

        if user.selected_character == None {
            user.selected_character = new_character.id;
            db::users::update(db_connection, &user)?;

            extra_text = "(and selected as default)";
        }

        let reply = CreateReply::default()
            .content(format!(
                "Character {character_name_stringified} created successfully! {extra_text}"
            ))
            .ephemeral(true);

        let _ = placeholder_message.edit(ctx, reply).await;
    } else {
        let reply = CreateReply::default()
            .content(format!("Error: No character name detected"))
            .ephemeral(true);

        let _ = placeholder_message.edit(ctx, reply).await;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn delete_character(ctx: Context<'_>, character_id: i32) -> Result<(), Error> {
    let db_connection = &mut db::establish_connection();

    let author = &ctx.author();
    let user_id = author.id.get();

    let is_admin = common::check_admin(
        ctx,
        ctx.guild_id().expect("No guild ID found???!??!?!?"),
        author.id,
    )
    .await;

    if is_admin {
        db::characters::delete_global(db_connection, character_id)?; //TODO if the bot goes public, this needs to also filter by guild
                                                                     // (don't let people delete other guilds' characters just because they're admin in their own one.)
                                                                     // This could even be done via the check admin function, if provided with the guild ID the character belongs to instead
                                                                     // of the current guild ID (not currently an issue as I whitelist which guilds the bot can join anyway)
    } else {
        db::characters::delete(db_connection, character_id, user_id)?;
    }

    let reply =
        CreateReply::default().content(format!("Succesfully deleted character id {character_id}"));

    let _ = ctx.send(reply).await;

    Ok(())
}
#[poise::command(slash_command, prefix_command)]
pub async fn select_character(ctx: Context<'_>, character_id: i32) -> Result<(), Error> {
    let placeholder = ctx
        .send(
            CreateReply::default()
                .content("Thinking... please wait")
                .ephemeral(true),
        )
        .await?;

    let db_connection = &mut db::establish_connection();

    let author = &ctx.author();
    let user_id = author.id.get();
    let mut user = db::users::get_or_create(db_connection, user_id)?;

    let character = db::characters::get(db_connection, character_id);

    match character {
        Ok((char)) => {
            let comparison_user_id = Some(user.id.clone());

            if char.user_id.eq(&comparison_user_id) {
                user.selected_character = Some(character_id);
                db::users::update(db_connection, &user)?;
                let char_name = char.name.unwrap_or("No Name".to_string());

                placeholder
                    .edit(
                        ctx,
                        CreateReply::default()
                            .content(format!("Selected character {char_name}"))
                            .ephemeral(true),
                    )
                    .await?;
            } else {
                placeholder
                    .edit(
                        ctx,
                        CreateReply::default()
                            .content(format!("Error: That's not your character!"))
                            .ephemeral(true),
                    )
                    .await?;
            }

            return Ok(());
        }

        Err(e) => {
            match e {
                db::DbError::NotFound => {
                    // Handle specific error
                    println!("Caught NotFound error");

                    placeholder.edit(
                        ctx,
                        CreateReply::default()
                            .content("You don't have a character with that id. Please do /get_characters to list your character sheets.")
                            .ephemeral(true),
                    ).await?;

                    return Ok(());
                }
                _ => {
                    return Err(Box::new(e));
                }
            }
        }
    }
}

#[poise::command(context_menu_command = "Set Spell Message")]
pub async fn set_spells(ctx: Context<'_>, msg: crate::serenity::Message) -> Result<(), Error> {
    let author = &ctx.author();
    let user_id = author.id.get();
    let db_connection = &mut db::establish_connection();
    let user = db::users::get_or_create(db_connection, user_id)?;

    if let Some(character_id) = user.selected_character {
        let mut character = db::characters::get(db_connection, character_id)?;

        character.spell_block_message_id = Some(msg.id.to_string());
        character.spell_block_channel_id = Some(msg.channel_id.to_string());

        db::characters::update(db_connection, &character)?;

        ctx.reply("Set your spell block successfully").await?;
    } else {
        return Err(Box::new(
            crate::stat_puller::StatPullerError::NoCharacterSheet,
        ));
    }

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
        let character_id = character.id.unwrap_or(-1).to_string();
        let channel_id = character
            .stat_block_channel_id
            .unwrap_or("No channel ID".to_string());

        character_messages.push(format!(
            "- {character_id}: {character_name} (in channel <#{channel_id}>)"
        ))
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
    let original_stat_block_message = ctx
        .send(
            CreateReply::default()
                .content("Thinking... please wait")
                .ephemeral(true),
        )
        .await?;

    let msg = ctx.say("*Thinking, please wait...*").await?;

    let (response_message, stat_message_raw) = stat_puller::get_stat_block_json(&ctx).await?;

    let stats: Value = serde_json::from_str(&response_message)?;

    let reply = CreateReply::default()
        .content(format!(
            "Original stat block text:\n```{}```",
            stat_message_raw
        ))
        .ephemeral(true);

    let _ = original_stat_block_message.edit(ctx, reply).await;

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
