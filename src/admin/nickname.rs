use poise::serenity_prelude::EditMember;

use crate::common::Context;
use crate::common::Error;
use crate::llm::BadKind;
use crate::serenity::Member;

#[poise::command(slash_command, prefix_command)]
pub async fn set_nick(ctx: Context<'_>, mut user: Member, nickname: String) -> Result<(), Error> {
    let perms = ctx
        .author_member()
        .await
        .expect("User should be a member of the server they sent the command in")
        .permissions(&ctx)?;

    if perms.manage_nicknames() {
        if !crate::llm::contains_badness(&nickname, &vec![BadKind::Hate]).await? {
            user.edit(ctx, EditMember::new().nickname(nickname)).await?;

            let author_id = &ctx.author().id;

            let target_id = &user.user.id;

            ctx.say(format!(
                "<@{author_id}> Set nickname of <@{target_id}> successfully"
            ))
            .await?;
        } else {
            ctx.say("Inappropriate name").await?;
        }
    } else {
        ctx.say("No manage nickname.").await?;
    }

    Ok(())
}
