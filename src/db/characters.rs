use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::db::*;

#[allow(dead_code)]
pub fn create(connection: &mut SqliteConnection, character: &Character) -> Result<(), DbError> {
    println!("Creating character");

    let _ = diesel::insert_into(schema::characters::table)
        .values(character)
        .execute(connection);

    return Ok(());
}

pub fn delete(
    connection: &mut SqliteConnection,
    character_id: i32,
    owner_id: u64,
) -> Result<(), DbError> {
    use self::schema::characters::dsl::*;

    diesel::delete(
        characters
            .filter(id.eq(character_id))
            .filter(user_id.eq(owner_id.to_string())),
    )
    .execute(connection);

    Ok(())
}

pub fn delete_global(connection: &mut SqliteConnection, character_id: i32) -> QueryResult<usize> {
    use self::schema::characters::dsl::*;

    diesel::delete(characters.filter(id.eq(character_id))).execute(connection)
}

#[allow(dead_code)]
pub fn get_from_user_id(
    connection: &mut SqliteConnection,
    user: u64,
) -> Result<Vec<Character>, DbError> {
    use self::schema::characters::dsl::*;

    let characters_result = characters
        .filter(user_id.eq(user.to_string()))
        .select(Character::as_select())
        .load(connection)
        .expect("Error loading posts");

    if characters_result.len() > 0 {
        Ok(characters_result)
    } else {
        Err(DbError::NotFound)
    }
}

#[allow(dead_code)]
pub fn get_latest(connection: &mut SqliteConnection, user: u64) -> Result<Character, DbError> {
    use self::schema::characters::dsl::*;

    let mut characters_result = characters
        .filter(user_id.eq(user.to_string())) // Filter by user_id
        .order(id.desc()) // Order by ID in descending order
        .limit(1)
        .select(Character::as_select())
        .load(connection)
        .expect("Error loading character");

    if characters_result.len() > 0 {
        let character = characters_result.remove(0);
        Ok(character)
    } else {
        Err(DbError::NotFound)
    }
}

#[allow(dead_code)]
pub fn get(connection: &mut SqliteConnection, character_id: i32) -> Result<Character, DbError> {
    use self::schema::characters::dsl::*;

    let mut characters_result = characters
        .filter(id.eq(Some(character_id)))
        .limit(1)
        .select(Character::as_select())
        .load(connection)
        .expect("Error loading characters");

    if characters_result.len() > 0 {
        let character = characters_result.remove(0);
        Ok(character)
    } else {
        Err(DbError::NotFound)
    }
}
#[allow(dead_code)]
pub fn get_by_char_sheet(
    connection: &mut SqliteConnection,
    channel_id: u64,
    message_id: u64,
) -> Result<Character, DbError> {
    use self::schema::characters::dsl::*;

    let mut characters_result = characters
        .filter(stat_block_channel_id.eq(channel_id.to_string()))
        .filter(stat_block_message_id.eq(message_id.to_string()))
        .limit(1)
        .select(Character::as_select())
        .load(connection)
        .expect("Error loading characters");

    if characters_result.len() > 0 {
        let character = characters_result.remove(0);
        Ok(character)
    } else {
        Err(DbError::NotFound)
    }
}

pub fn update(connection: &mut SqliteConnection, character: &Character) -> Result<(), DbError> {
    use self::schema::characters::dsl::*;

    let character_id = &character.id;

    // println!("Updating character {character_id}");

    let _ = diesel::update(characters.filter(id.eq(character_id)))
        .set(character)
        .execute(connection);

    return Ok(());
}

// #[allow(dead_code)]
// pub fn update(connection: &mut SqliteConnection, character: &Character) -> Result<(), DbError> {
//     println!("Updating character");

//     let character = diesel::update(characters.find(id))
//         .set(count.eq(character.count))
//         .execute(connection);

//     return Ok(());
// }
