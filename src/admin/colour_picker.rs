use crate::common::Context;
use crate::common::Error;

use crate::serenity::Colour;
use crate::serenity::EditRole;

#[poise::command(slash_command, prefix_command)]
pub async fn set_colour(ctx: Context<'_>, name_colour: String) -> Result<(), Error> {
    let role_name = ctx.author().id.to_string();
    let colour = Colour::from(u32::from_str_radix(&name_colour.replace("#", ""), 16)?);
    let guild_id = ctx.guild_id().ok_or("No guild")?;
    let http = ctx.serenity_context().http.clone();

    let roles = ctx.guild().expect("No guild").roles.clone();

    if let Some(role) = roles.values().find(|r| r.name == role_name) {
        guild_id.delete_role(&http, role.id).await?;
    }

    let role = guild_id
        .create_role(&http, EditRole::new().name(role_name).colour(colour))
        .await?;

    let user = ctx.author_member().await.ok_or("No member")?;
    user.add_role(&http, role.id).await?;

    ctx.say(format!("Set your colour to {name_colour} successfully"))
        .await?;

    Ok(())
}
