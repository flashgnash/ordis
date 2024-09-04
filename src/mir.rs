use crate::common::safe_to_number;
use crate::common::Context;
use crate::common::Error;
use crate::db;
use crate::dice::roll_replace;
use poise::CreateReply;
use serde_json::Value;

use crate::dice::roll_internal;

use crate::stat_puller::get_stat_block_json;

#[poise::command(slash_command, prefix_command)]
pub async fn stat_roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    let result = roll_internal(&dice).await?;

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

#[poise::command(slash_command, prefix_command)]
pub async fn level_up(ctx: Context<'_>, num_levels: i32) -> Result<(), Error> {
    let (response_message, stat_message_raw) = get_stat_block_json(&ctx).await?;

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
        let (hit_die_result, _) = roll_replace(&hit_die.as_str())?;
        let (stat_die_result, _) = roll_replace(&stat_die.as_str())?;
        let (spell_die_result, _) = roll_replace(&spell_die.as_str())?;

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
