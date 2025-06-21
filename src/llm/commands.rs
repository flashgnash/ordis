#[poise::command(slash_command, prefix_command)]
pub async fn ask(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response = generate_ordis(&message, None).await?;

    let response_message = &response.choices[0].message.content;

    println!("{}", response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(slash_command, prefix_command)]
pub async fn draw(ctx: Context<'_>, message: String) -> Result<(), Error> {
    let msg = ctx.say("*Thinking, please wait...*").await?;

    let response_message = generate_image(&message).await?;

    println!("Generated image URL: {}", response_message);

    let reply = CreateReply::default().content(response_message);

    msg.edit(ctx, reply).await?;

    return Ok(());
}

#[poise::command(slash_command, prefix_command)]
pub async fn translate(ctx: Context<'_>, message: String) -> Result<(), Error> {
    translate_internal(ctx, message).await?;

    return Ok(());
}
