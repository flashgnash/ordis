pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn fetch_message_poise<E>(
    ctx: &poise::Context<'_, Data, E>,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<poise::serenity_prelude::Message, Error> {
    Ok(ctx.http().get_message(channel_id, message_id).await?)
}

pub async fn fetch_message(
    ctx: &poise::serenity_prelude::Context,
    channel_id: poise::serenity_prelude::ChannelId,
    message_id: poise::serenity_prelude::MessageId,
) -> Result<poise::serenity_prelude::Message, Error> {
    Ok(ctx.http.get_message(channel_id, message_id).await?)
}

pub fn strip_non_numerical(s: &str) -> String {
    s.chars().filter(|c| c.is_digit(10)).collect()
}

pub fn safe_to_number(s: &str) -> i32 {
    let part_stripped = strip_non_numerical(s);

    if part_stripped.len() == 0 {
        return 0;
    }

    return part_stripped.parse::<i32>().unwrap();
}

// &[i32] is a slice reference (which means it doesn't borrow the variable)
pub fn join_to_string(numbers: &[i32], separator: &str) -> String {
    let s: String = numbers
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(separator);
    return s;
}

pub fn sum_array(arr: &Vec<i32>) -> i32 {
    let mut result = 0;

    for num in arr {
        result = result + num;
    }
    return result;
}
