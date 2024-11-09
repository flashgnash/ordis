pub mod spell_sheet;
pub mod stat_block;

use lazy_static::lazy_static;
use poise::serenity_prelude::CreateEmbed;
use tokio::sync::Mutex;

use super::spells;
use super::spells::ManaSpellResource;
use super::spells::Spell;
use super::spells::SpellResource;
use super::spells::SpellType;

use spell_sheet::SpellSheet;
use stat_block::StatBlock;

use std::collections::HashMap;

use crate::common;
use crate::common::safe_to_number;
use crate::common::Context;
use crate::common::Error;
use crate::db;
use crate::db::models::Character;

use diesel::SqliteConnection;

use super::get_user_character;
use super::CharacterSheetable;
use super::RpgError;

use crate::dice;

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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;

    let stat_block: Value = serde_json::from_str(
        &stat_block
            .sheet_info
            .jsonified_message
            .expect("Stat block should always generate json"),
    )?;

    let spell_block_result: SpellSheet = super::get_sheet(&ctx).await?;

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

static BAR_LENGTH: i32 = 17;

#[poise::command(slash_command, prefix_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;
    let db_connection = &mut db::establish_connection();

    let bar_length = BAR_LENGTH;

    let character = get_user_character(&ctx, db_connection).await?;

    let character_name = &character.name.clone().unwrap_or("No name?".to_string());
    let character_channel = &character
        .stat_block_channel_id
        .clone()
        .unwrap_or("".to_string());

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &character, &bar_length);

    let max_health = stat_block.max_hp;
    let health = stat_block.hp;

    let mut health_message_content = "Current health unknown.".to_string();

    if let Some(max_health) = max_health {
        let health = health.unwrap_or(max_health);
        health_message_content = format!(
            "♥️ {} ``{health} / {max_health}``",
            crate::common::draw_bar(
                health as i32,
                max_health as i32,
                BAR_LENGTH as usize,
                "🟥",
                "⬛"
            )
        );
    }

    let mut hunger_message_content = "Hunger unknown.".to_string();
    if let Some(hunger) = stat_block.hunger {
        hunger_message_content = format!(
            "🍖 {} ``{hunger} / 10``",
            crate::common::draw_bar(hunger as i32, 10, BAR_LENGTH as usize, "🟨", "⬛")
        );
    }

    let mut active_spells_content: String = "".to_string();

    let active_spells_map = ACTIVE_SPELLS.lock().await;

    if let Some(active_spells) =
        active_spells_map.get(&character.id.expect("Character ID should never be null"))
    {
        active_spells_content = "Active Spells:\n".to_string();

        let mut total_mana_diff: ManaSpellResource = ManaSpellResource { mana: 0 };

        for spell in active_spells {
            active_spells_content = active_spells_content
                + &format!(
                    "- {}: {} per turn\n",
                    &spell.name.clone().unwrap_or("No spell name".to_string()),
                    &spell
                        .cost
                        .clone()
                        .and_then(|c| Some(c.to_string()))
                        .unwrap_or("No mana cost".to_string())
                );

            if let Some(mana_change) = &spell.cost {
                total_mana_diff.add(&mana_change).mana;
            }
        }

        active_spells_content =
            active_spells_content + &format!("\nNet mana change: {total_mana_diff} per turn");
    }

    let embed = CreateEmbed::default().title(format!("{character_name}\n<#{character_channel}>\n​\n")).description(format!(
                "\n\n{health_message_content}\n\n{mana_message_content}\n\n{hunger_message_content}\n​\n{active_spells_content}\n"
            ));
    // embed.description("Test");

    placeholder_message
        .edit(
            ctx,
            CreateReply::default()
                .content("")
                .embed(embed)
                .ephemeral(true),
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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &character, &BAR_LENGTH);

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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character, &BAR_LENGTH);

    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    placeholder_message
        .edit(ctx, CreateReply::default().content(mana_message_content))
        .await?;

    Ok(())
}

fn get_mana_bar_message(stat_block: &StatBlock, character: &Character, bar_length: &i32) -> String {
    return format!(
        "🪄 {} ``{} / {}`` ",
        crate::common::draw_bar(
            character.mana.unwrap_or(0),
            stat_block.energy_pool.unwrap_or(0) as i32,
            BAR_LENGTH as usize,
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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let mana_message_content = format!(
        "Current Energy: \n\n{}",
        get_mana_bar_message(&stat_block, &character, &BAR_LENGTH)
    );

    if let (Some(channel_id), Some(message_id)) = (
        &character.mana_readout_channel_id,
        &character.mana_readout_message_id,
    ) {
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
    return Err(Box::new(RpgError::NoSpellSheet));
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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character, &BAR_LENGTH);

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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character, &BAR_LENGTH);

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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let mana_message_content = get_mana_bar_message(&stat_block, &modified_character, &BAR_LENGTH);

    placeholder_message
        .edit(ctx, CreateReply::default().content(mana_message_content))
        .await?;

    Ok(())
}

lazy_static! {
    static ref ACTIVE_SPELLS: Mutex<HashMap<i32, Vec<Spell<ManaSpellResource>>>> =
        Mutex::new(HashMap::new());
}

#[poise::command(slash_command, prefix_command)]
pub async fn end_turn(ctx: Context<'_>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    let db_connection = &mut db::establish_connection();
    let character = get_user_character(&ctx, db_connection).await?;

    let active_spells_map = ACTIVE_SPELLS.lock().await;

    if let Some(active_spells) =
        active_spells_map.get(&character.id.expect("Character ID should never be null"))
    {
        let stat_block: StatBlock = super::get_sheet(&ctx).await?;
        let max_mana = stat_block.energy_pool;

        for spell in active_spells.into_iter() {
            let mut name = "unknown spell name";

            if let Some(spell_name) = &spell.name {
                name = spell_name;
            }

            let mut cur_mana = max_mana.ok_or(RpgError::NoMaxEnergy)? as i32;

            if let Some(mana) = character.mana {
                cur_mana = mana;
            }

            let new_mana = cur_mana + spell.cost.as_ref().ok_or(RpgError::NoSpellCost)?.mana;

            if new_mana >= 0 {
                let modified_char =
                    set_mana_internal(ctx, db_connection, character.clone(), new_mana).await?;

                let cast_time = &spell
                    .cast_time
                    .as_ref()
                    .and_then(|s| Some(s.to_string()))
                    .unwrap_or("No cast time found".to_string());

                ctx.reply(format!(
                    "{} casts **{name}**: (Cast time: {cast_time})\n",
                    &modified_char
                        .name
                        .as_ref()
                        .unwrap_or(&"Unknown name".to_string())
                ))
                .await?;

                // ctx.say(format!("Casting spell {}", name)).await?;
            } else {
                ctx.say(format!("Spell {} failed due to lack of mana", name))
                    .await?;
            }
        }

        placeholder_message
            .edit(ctx, CreateReply::default().content("Turn ended"))
            .await?;
    } else {
        placeholder_message
            .edit(ctx, CreateReply::default().content("No active spells"))
            .await?;
    }

    Ok(())
}
#[poise::command(slash_command, prefix_command)]
pub async fn cast_spell(ctx: Context<'_>, spell_name: String) -> Result<(), Error> {
    let placeholder = CreateReply::default().content("*Thinking, please wait...*");
    let placeholder_message = ctx.send(placeholder).await?;

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;
    let spell_sheet: SpellSheet = super::get_sheet(&ctx).await?;

    let max_mana = stat_block.energy_pool;

    let spells = spell_sheet.spells.ok_or(RpgError::NoSpellSheet)?;

    if let Some(spell) = spells.get(&spell_name) {
        let spell_cost = spell.cost.clone().ok_or(RpgError::NoSpellCost)?;

        // let spell_cost_string = spell_cost.to_string();

        let mut spell_cost_word = "Cost";
        if spell_cost.mana > 0 {
            spell_cost_word = "Gain";
        }

        let spell_type = spell.spell_type.as_ref().unwrap_or(&SpellType::Unknown);

        let cast_time = &spell
            .cast_time
            .as_ref()
            .and_then(|s| Some(s.to_string()))
            .unwrap_or("No cast time found".to_string());

        let spell_name = common::capitalize_first_letter(&spell_name);

        let db_connection = &mut db::establish_connection();

        let character = get_user_character(&ctx, db_connection).await?;

        let mana = character
            .mana
            .unwrap_or(max_mana.ok_or(RpgError::NoMaxEnergy)? as i32);

        let mut modified_char: Option<Character> = None;

        match spell_type {
            SpellType::Single => {
                let new_mana = mana + spell.cost.clone().ok_or(RpgError::NoSpellCost)?.mana;

                if new_mana >= 0 {
                    modified_char =
                        Some(set_mana_internal(ctx, db_connection, character, new_mana).await?);

                    placeholder_message
                        .edit(
                            ctx,
                            CreateReply::default().content(format!(
                                "{} casts **{spell_name}**: (Cast time: {cast_time})\n",
                                &modified_char
                                    .as_ref()
                                    .expect("Just set this")
                                    .name
                                    .as_ref()
                                    .unwrap_or(&"Unknown name".to_string())
                            )),
                        )
                        .await?;
                } else {
                    placeholder_message
                        .edit(
                            ctx,
                            CreateReply::default().content(format!(
                                "Failed to cast **{spell_name}** (not enough mana)\n",
                            )),
                        )
                        .await?;
                }
            }
            SpellType::Toggle => {
                let mut active_spells_map = ACTIVE_SPELLS.lock().await;

                let character_id = character.id.expect("Character ID should never be null");

                if !active_spells_map.contains_key(&character_id) {
                    println!("Inserting new active spell list for {}", ctx.author().name);
                    active_spells_map.insert(character_id, Vec::new());
                }

                let mut is_spell_active: bool = false;

                let active_spells = active_spells_map
                    .get(&character.id.expect("Character ID should never be null"))
                    .expect("Just set this");

                for active_spell in active_spells {
                    if &spell.name == &active_spell.name {
                        let msg = format!(
                            "{} disabled spell {}",
                            &ctx.author().name,
                            &spell.name.clone().unwrap_or("Unnamed spell".to_string())
                        );
                        ctx.reply(msg).await?;

                        is_spell_active = true;
                    }
                }

                if !is_spell_active {
                    let active_spells_mut = active_spells_map
                        .get_mut(&character.id.expect("Character ID should never be null"))
                        .expect("Just set this");

                    active_spells_mut.push((*spell).clone());

                    let msg = format!(
                        "{} enabled spell {}",
                        &ctx.author().name,
                        &spell.name.clone().unwrap_or("Unnamed spell".to_string())
                    );
                    ctx.reply(msg).await?;
                }
            }
            SpellType::Summon => {
                ctx.reply("Summon spells are currently not supported")
                    .await?;
            }
            _ => {
                ctx.reply("Unknown spell type").await?;
            }
        }

        if let Some(modified_char) = modified_char {
            let mana_message_content =
                get_mana_bar_message(&stat_block, &modified_char, &BAR_LENGTH);

            ctx.send(
                CreateReply::default()
                    .content(format!("New mana: \n{mana_message_content}"))
                    .ephemeral(true),
            )
            .await?;
        }
    } else {
        let mut spell_list_message = "Spell not found. Available Spells: \n".to_string();

        for (spell_name, _) in spells {
            spell_list_message += &format!("- {spell_name} \n");
        }

        let spell_list_reply = CreateReply::default()
            .content(spell_list_message)
            .ephemeral(true);

        ctx.send(spell_list_reply).await?;
        // return Err(Box::new(crate::stat_puller::RpgError::SpellNotFound));
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn list_spells(ctx: Context<'_>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);
    let placeholder_message = ctx.send(placeholder).await?;

    let spell_sheet: SpellSheet = super::get_sheet(&ctx).await?;

    let spells = spell_sheet.spells.ok_or(RpgError::NoSpellSheet)?;

    let mut spell_list_message = "Available Spells: \n".to_string();

    for (spell_name, _) in spells {
        spell_list_message += &format!("- {spell_name} \n");
    }

    let spell_list_reply = CreateReply::default()
        .content(spell_list_message)
        .ephemeral(true);

    placeholder_message.edit(ctx, spell_list_reply).await?;
    // return Err(Box::new(crate::stat_puller::RpgError::SpellNotFound));

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, dice_expression: Option<String>) -> Result<(), Error> {
    let placeholder = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    _ = ctx.send(placeholder).await?;

    let stat_block_result: Result<StatBlock, Error> = super::get_sheet(&ctx).await;

    //TODO make this default configurable per server
    let mut dice = dice_expression
        .unwrap_or("1d100".to_string())
        .to_lowercase();

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
                    if let Some(int_value) = value.as_i64() {
                        let stat_mod = int_value / 10;
                        str_replaced = str_replaced.replace(stat, &stat_mod.to_string());
                    }
                }
            }
        }

        Err(e) => {
            if let Some(stat_puller_error) = e.downcast_ref::<RpgError>() {
                match stat_puller_error {
                    RpgError::NoCharacterSelected | RpgError::NoCharacterSheet => {
                        // Handle specific error
                        println!("Caught NoCharacterSheet error");

                        nag_user_about_character_sheet = true;
                    }
                    _ => return Err(e), // Propagate other RpgError variants
                }
            } else {
                // Propagate other errors that are not RpgError
                return Err(e);
            }
        }
    }

    let result = dice::roll_internal(&str_replaced).await?;

    let author = ctx.author();

    let mut nick = author.name.to_string();

    if let Some(guild_id) = ctx.guild_id() {
        let author_nick = author.nick_in(ctx, guild_id).await;

        if let Some(author_nick) = author_nick {
            nick = author_nick;
        }
    }

    dice::output_roll_message(ctx, result, nick).await?;

    if nag_user_about_character_sheet {
        let character_sheet_missing_message = CreateReply::default()
                            .content("Hint: if you configure and select a character sheet you can add stat modifiers to your rolls (e.g /roll 1d100+str )")
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

    let response_message = StatBlock::from_message(&ctx.serenity_context(), msg.channel_id, msg.id)
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

    let mut user = db::users::get(db_connection, user_id)?;
    if user.selected_character == Some(character_id) {
        println!("Removing selected character");
        db::users::unset_character(db_connection, &user)?;
    }

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

            if char.user_id.eq(&comparison_user_id)
                || common::check_admin(
                    ctx,
                    ctx.guild_id().ok_or(RpgError::NoGuildId)?,
                    ctx.author().id,
                )
                .await
            {
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
        return Err(Box::new(RpgError::NoCharacterSheet));
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

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;

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

    let energy_die = stats
        .get("energy_die_per_level")
        .ok_or(RpgError::NoEnergyDie)?
        .to_string();
    let magic_die = stats
        .get("magic_die_per_level")
        .ok_or(RpgError::NoMagicDie)?
        .to_string();
    let training_die = stats
        .get("training_die_per_level")
        .ok_or(RpgError::NoTrainingDie)?
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

#[poise::command(slash_command)]
pub async fn pull_stats(ctx: Context<'_>) -> Result<(), Error> {
    let thinking_message = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    let msg = ctx.send(thinking_message).await?;

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;

    let reply = CreateReply::default().content(
        stat_block
            .sheet_info
            .jsonified_message
            .expect("Stat block should always generate json"),
    );
    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(
    slash_command,
    // description_localized = "Pull a single stat from your character sheet"
)]
pub async fn pull_stat(ctx: Context<'_>, stat_name: String) -> Result<(), Error> {
    let stat_block_thinking_message = CreateReply::default()
        .content("*Thinking, please wait...*")
        .ephemeral(true);

    let msg = ctx.send(stat_block_thinking_message).await?;

    // let stat_message = fetch_message_poise(&ctx, channel_id, message_id).await?;

    let stat_block: StatBlock = super::get_sheet(&ctx).await?;

    let stats: Value = serde_json::from_str(
        &stat_block
            .sheet_info
            .jsonified_message
            .expect("Stat block should always generate json"),
    )?;

    // println!("```json\n{}```", response_message);

    let reply = CreateReply::default().content(stats.get(stat_name).unwrap().to_string());
    msg.edit(ctx, reply).await?;

    return Ok(());
}
