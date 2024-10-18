pub mod stat_puller;

pub mod spell_sheet;
pub mod stat_block;

use spell_sheet::SpellSheet;
use spell_sheet::SpellType;
use stat_block::StatBlock;
use stat_puller::get_sheet;
use stat_puller::get_user_character;
use stat_puller::CharacterSheetable;

use crate::common;
use crate::common::safe_to_number;
use crate::common::Context;
use crate::common::Error;
use crate::db;
use crate::db::models::Character;

use diesel::SqliteConnection;

use crate::dice;
use stat_puller::StatPullerError;

use poise::serenity_prelude::CreateMessage;
use poise::serenity_prelude::EditMessage;
use poise::CreateReply;
use serde_json::Value;

use regex::Regex;

use poise::serenity_prelude::ChannelId;
use poise::serenity_prelude::MessageId;

#[poise::command(slash_command, prefix_command)]
pub async fn pull_spellsheet(ctx: Context<'_>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    let placeholder_msg = ctx.send(placeholder).await?;

    let stat_block: StatBlock = stat_puller::get_sheet(&ctx).await?;

    let stat_block: Value = serde_json::from_str(
        &stat_block
            .sheet_info
            .jsonified_message
            .expect("Stat block should always generate json"),
    )?;

    let spell_block_result: SpellSheet = stat_puller::get_sheet(&ctx).await?;

    let spell_block: Value = serde_json::from_str(
        &spell_block_result
            .sheet_info
            .jsonified_message
            .expect("SpellBlock should always have json generated on construction"),
    )?;

    placeholder_msg
        .edit(
            ctx,
            CreateReply::default()
                .content(spell_block.to_string())
                .ephemeral(true),
        )
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;
    let db_connection = &mut db::establish_connection();

    let character = get_user_character(&ctx, db_connection).await?;

    let character_name = &character.name.clone().unwrap_or("No name?".to_string());
    let character_channel = &character
        .stat_block_channel_id
        .clone()
        .unwrap_or("".to_string());

    let stat_block = StatBlock::from_character_with_cache(&ctx, &character).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &character);

    let max_health = stat_block.max_hp;
    let health = stat_block.hp;

    let mut health_message_content = "Current health unknown.".to_string();

    if let Some(max_health) = max_health {
        let health = health.unwrap_or(max_health);
        health_message_content = format!(
            "♥️ ``{} {health}/{max_health}``",
            crate::common::draw_bar(health as i32, max_health as i32, 13, "🟥", "⬛")
        );
    }

    let mut hunger_message_content = "Hunger unknown.".to_string();
    if let Some(hunger) = stat_block.hunger {
        hunger_message_content = format!(
            "🍖 ``{} {hunger} / 10``",
            crate::common::draw_bar(hunger as i32, 10, 13, "🟨", "⬛")
        );
    }

    placeholder_message
        .edit(
            ctx,
            CreateReply::default().content(format!(
                "**{character_name}** (<#{character_channel}>)\n\n{health_message_content}\n\n{mana_message_content}\n\n{hunger_message_content}\n\n**📔 Active spells**\n> - Invisibility: -50/turn\n> - Meditation: +25/turn\n> Net: -25/turn"
            )),
        )
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn get_mana(ctx: Context<'_>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;
    let db_connection = &mut db::establish_connection();

    let character = get_user_character(&ctx, db_connection).await?;

    let stat_block = StatBlock::from_character_with_cache(&ctx, &character).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &character);

    placeholder_message
        .edit(ctx, CreateReply::default().content(mana_message_content))
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn set_mana(ctx: Context<'_>, mana: i32) -> Result<(), Error> {
    let db_connection = &mut db::establish_connection();

    let character = get_user_character(&ctx, db_connection).await?;

    let old_mana = character.mana.unwrap_or(0);

    let modified_character = set_mana_internal(ctx, db_connection, character, mana).await?;

    ctx.reply(format!(
        "Modified energy. Change: {}",
        modified_character.mana.unwrap_or(0) - old_mana
    ))
    .await?;

    let stat_block = StatBlock::from_character_with_cache(&ctx, &modified_character).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character);

    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    placeholder_message
        .edit(ctx, CreateReply::default().content(mana_message_content))
        .await?;

    Ok(())
}

fn get_mana_bar_message(stat_block: &StatBlock, character: &Character) -> String {
    return format!(
        "🪄 ``{} {} / {}`` ",
        crate::common::draw_bar(
            character.mana.unwrap_or(0),
            stat_block.energy_pool.unwrap_or(0) as i32,
            13,
            "🟦",
            "⬛"
        ),
        character.mana.unwrap_or(0),
        stat_block.energy_pool.unwrap_or(0),
    );
}

async fn update_mana_readout(
    ctx: Context<'_>,
    character: &Character,
    db_connection: &mut SqliteConnection,
) -> Result<Character, Error> {
    let mut modified_character = character.clone();

    let stat_block = StatBlock::from_character_with_cache(&ctx, character).await?;

    let mana_message_content = format!(
        "Current Energy: \n\n{}",
        get_mana_bar_message(&stat_block, &character)
    );

    if let (Some(channel_id), Some(message_id)) = (
        &character.mana_readout_channel_id,
        &character.mana_readout_message_id,
    ) {
        println!("Using existing gauge");
        let channel: ChannelId = channel_id.parse()?;
        let message: MessageId = message_id.parse()?;

        let edit_result = channel
            .edit_message(
                ctx,
                message,
                EditMessage::default().content(&mana_message_content),
            )
            .await;

        match edit_result {
            Ok(_) => {
                return Ok(modified_character);
            }
            Err(_) => {
                println!("Message was deleted. Clearing saved message ID and trying again");
                modified_character.mana_readout_channel_id = None;

                // db::characters::update(db_connection, &modified_character);

                // return update_mana_readout(ctx, &modified_character, db_connection).await;
            }
        };
    };

    if let Some(channel_id) = &character.spell_block_channel_id {
        println!("Making new gauge in channel {channel_id}");

        modified_character.mana_readout_channel_id = Some(channel_id.to_string());

        let channel: ChannelId = channel_id.parse().unwrap();

        let message = channel
            .send_message(ctx, CreateMessage::default().content(&mana_message_content))
            .await?;

        modified_character.mana_readout_message_id = Some(message.id.to_string());

        db::characters::update(db_connection, &modified_character)?;

        return Ok(modified_character);
    }

    println!("No channel id or message id - no spell message?");
    return Err(Box::new(StatPullerError::NoSpellSheet));
}

async fn set_mana_internal(
    ctx: Context<'_>,
    db_connection: &mut SqliteConnection,
    character: Character,
    mana: i32,
) -> Result<Character, Error> {
    let mut new_character = character.clone();

    new_character.mana = Some(mana);

    db::characters::update(db_connection, &new_character)?;

    update_mana_readout(ctx, &new_character, db_connection).await?;

    Ok(new_character)
}

#[poise::command(slash_command, prefix_command)]
pub async fn add_mana(ctx: Context<'_>, modifier: i32) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    let db_connection = &mut db::establish_connection();

    let character = get_user_character(&ctx, db_connection).await?;

    let old_mana = character.mana.unwrap_or(0);

    let calc_result = old_mana + modifier;

    let modified_character = set_mana_internal(ctx, db_connection, character, calc_result).await?;

    ctx.reply(format!(
        "Modified energy. Change: {}",
        modified_character.mana.unwrap_or(0) - old_mana
    ))
    .await?;

    let stat_block = StatBlock::from_character_with_cache(&ctx, &modified_character).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character);

    placeholder_message
        .edit(ctx, CreateReply::default().content(mana_message_content))
        .await?;

    Ok(())
}
#[poise::command(slash_command, prefix_command)]
pub async fn sub_mana(ctx: Context<'_>, modifier: i32) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    let db_connection = &mut db::establish_connection();

    let character = get_user_character(&ctx, db_connection).await?;

    let old_mana = character.mana.unwrap_or(0);

    let calc_result = old_mana - modifier;

    let modified_character = set_mana_internal(ctx, db_connection, character, calc_result).await?;

    // character.mana = Some(calc_result);

    // db::characters::update(db_connection, &character)?;

    ctx.reply(format!(
        "Modified energy. Change: {}",
        modified_character.mana.unwrap_or(0) - old_mana
    ))
    .await?;

    let stat_block = StatBlock::from_character_with_cache(&ctx, &modified_character).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character);

    placeholder_message
        .edit(ctx, CreateReply::default().content(mana_message_content))
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn mod_mana(ctx: Context<'_>, modifier: String) -> Result<(), Error> {
    if (!modifier.contains("n")) {
        ctx.reply("Your modification should include the letter N to represent your current energy (use add_mana if you just want to add or subtract)")
            .await?;

        return Ok(());
    }

    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    let db_connection = &mut db::establish_connection();

    let character = get_user_character(&ctx, db_connection).await?;

    let old_mana = character.mana.unwrap_or(0);

    let expression = modifier.replace("n", &character.mana.unwrap_or(0).to_string());

    let calc_result = meval::eval_str(expression)? as i32;

    let modified_character = set_mana_internal(ctx, db_connection, character, calc_result).await?;

    ctx.reply(format!(
        "Modified energy. Change: {}",
        modified_character.mana.unwrap_or(0) - old_mana
    ))
    .await?;

    let stat_block = StatBlock::from_character_with_cache(&ctx, &modified_character).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character);

    placeholder_message
        .edit(ctx, CreateReply::default().content(mana_message_content))
        .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn cast_spell(ctx: Context<'_>, spell_name: String) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    let stat_block: StatBlock = stat_puller::get_sheet(&ctx).await?;
    let spell_sheet: SpellSheet = stat_puller::get_sheet(&ctx).await?;

    let max_mana = stat_block.energy_pool;

    let spells = spell_sheet.spells.ok_or(StatPullerError::NoSpellSheet)?;

    if let Some(spell) = spells.get(&spell_name) {
        let spell_cost = spell.mana_change.ok_or(StatPullerError::NoSpellCost)?;

        let spell_cost_string = spell_cost.to_string();

        let mut spell_cost_word = "Cost";
        if spell_cost > 0 {
            spell_cost_word = "Gain";
        }

        let spell_type = spell.spell_type.as_ref().unwrap_or(&SpellType::Unknown);

        let cast_time = &spell
            .cast_time
            .as_ref()
            .and_then(|s| Some(s.to_string()))
            .unwrap_or("No cast time found".to_string());

        let spell_name = common::capitalize_first_letter(&spell_name);

        match spell_type {
            SpellType::Single => {
                let db_connection = &mut db::establish_connection();

                let character = get_user_character(&ctx, db_connection).await?;

                let mana = character
                    .mana
                    .unwrap_or(max_mana.ok_or(StatPullerError::NoMaxEnergy)? as i32);

                let new_mana = mana + spell.mana_change.ok_or(StatPullerError::NoSpellCost)? as i32;

                if new_mana >= 0 {
                    let modified_char =
                        set_mana_internal(ctx, db_connection, character, new_mana).await?;

                    ctx.reply(format!(
                        "{} casts **{spell_name}**: (Cast time: {cast_time})\n",
                        &modified_char
                            .name
                            .as_ref()
                            .unwrap_or(&"Unknown name".to_string())
                    ))
                    .await?;
                } else {
                    ctx.reply(format!(
                        "Failed to cast **{spell_name}** (not enough mana)\n",
                    ))
                    .await?;
                }
            }
            _ => {
                let msg = format!("Only single cast spells are supported right now, sorry!");

                ctx.reply(msg).await?;
            }
        }
    } else {
        let mut spell_list_message = "Spell not found. Available Spells: \n".to_string();

        for (spell_name, _) in spells {
            spell_list_message += &format!("- {spell_name} \n");
        }

        let spell_list_reply = CreateReply::default()
            .content(spell_list_message)
            .ephemeral(true);

        placeholder_message.edit(ctx, spell_list_reply).await?;
        // return Err(Box::new(crate::stat_puller::StatPullerError::SpellNotFound));
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice_expression: Option<String>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    _ = ctx.send(placeholder).await?;

    let stat_block_result: Result<StatBlock, Error> = get_sheet(&ctx).await;

    //TODO make this default configurable per server
    let mut dice = dice_expression.unwrap_or("1d100".to_string());

    //Replace d100 with 1d100, d6 with 1d6 etc

    let re = Regex::new(r"(^|[^\d])d(\d+)").unwrap();

    dice = re.replace_all(&dice, "1d$11d$2").to_string();

    // println!("{}", dice);

    let mut str_replaced = dice;

    let mut nag_user_about_character_sheet = false;

    match stat_block_result {
        Ok(stat_block) => {
            if let Some(stats_object) = stat_block
                .stats
                .as_ref()
                .and_then(|stats| stats.as_object())
            {
                for (stat, value) in stats_object {
                    println!("{stat}: {value}");
                    if let Some(int_value) = value.as_i64() {
                        let stat_mod = int_value / 10;
                        str_replaced = str_replaced.replace(stat, &stat_mod.to_string());
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

    dice::output_roll_messages(
        ctx,
        results,
        ctx.author()
            .nick_in(
                ctx,
                ctx.guild_id()
                    .expect("Tried to roll in non-guild - TODO remove this issue"),
            )
            .await
            .unwrap_or(ctx.author().name.to_string()),
    )
    .await?;

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

    let response_message = StatBlock::from_message(&ctx, msg.channel_id, msg.id)
        .await?
        .sheet_info
        .jsonified_message
        .expect("Stat block failed to construct");

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
        println!(
            "{}",
            &stats
                .get("name")
                .and_then(|n| Some(n.to_string()))
                .unwrap_or("name unknown".to_string())
        );

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

            mana: None,
            mana_readout_channel_id: None,
            mana_readout_message_id: None,
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
        Ok(char) => {
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
    let placeholder = ctx
        .send(
            CreateReply::default()
                .content("Thinking... please wait")
                .ephemeral(true),
        )
        .await?;

    let author = &ctx.author();
    let user_id = author.id.get();
    let db_connection = &mut db::establish_connection();
    let user = db::users::get_or_create(db_connection, user_id)?;

    if let Some(character_id) = user.selected_character {
        let mut character = db::characters::get(db_connection, character_id)?;

        character.spell_block_message_id = Some(msg.id.to_string());
        character.spell_block_channel_id = Some(msg.channel_id.to_string());

        db::characters::update(db_connection, &character)?;

        placeholder
            .edit(
                ctx,
                CreateReply::default().content("Set your spell block successfully"),
            )
            .await?;
    } else {
        return Err(Box::new(stat_puller::StatPullerError::NoCharacterSheet));
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

    let stat_block: StatBlock = stat_puller::get_sheet(&ctx).await?;

    let stats: Value = serde_json::from_str(
        &stat_block
            .sheet_info
            .jsonified_message
            .expect("Stat block should always generate json"),
    )?;

    let reply = CreateReply::default()
        .content(format!(
            "Original stat block text:\n```{}```",
            stat_block
                .sheet_info
                .original_message
                .expect("Stat block should always have an original message")
        ))
        .ephemeral(true);

    let _ = original_stat_block_message.edit(ctx, reply).await;

    println!("{}", stats);

    let energy_die = stats
        .get("energy_die_per_level")
        .ok_or(StatPullerError::NoEnergyDie)?
        .to_string();
    let magic_die = stats
        .get("magic_die_per_level")
        .ok_or(StatPullerError::NoMagicDie)?
        .to_string();
    let training_die = stats
        .get("training_die_per_level")
        .ok_or(StatPullerError::NoTrainingDie)?
        .to_string();

    let mut energy_die_sum: i32 = 0;
    let mut magic_die_sum: i32 = 0;
    let mut training_die_sum: i32 = 0;

    let mut message = format!(
        "Per Level: \nEnergy: {energy_die} \\| Magic: {magic_die} \\| Training: {training_die}\n\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\nRolls:"
    );

    for i in 1..num_levels + 1 {
        let (energy_die_result, _) = dice::roll_replace(&energy_die.as_str())?;
        let (magic_die_result, _) = dice::roll_replace(&magic_die.as_str())?;
        let (training_die_result, _) = dice::roll_replace(&training_die.as_str())?;

        energy_die_sum = energy_die_sum + safe_to_number(&energy_die_result);
        magic_die_sum = magic_die_sum + safe_to_number(&magic_die_result);
        training_die_sum = training_die_sum + safe_to_number(&training_die_result);

        message = format!(
            "{message}\n{i}.  ⚡️ {energy_die_result}    🐇 {magic_die_result}    🏋 {training_die_result}"
        );
    }
    message = message.replace('"', "");
    message = format!("{message}\n\n**Total**:\n    ⚡️ {energy_die_sum}    🐇 {magic_die_sum}    🏋 {training_die_sum}");
    let reply = CreateReply::default().content(message);
    msg.edit(ctx, reply).await?;

    return Ok(());
}
