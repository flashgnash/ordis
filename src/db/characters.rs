use crate::common::Error;
use crate::db::DbError;
use crate::db::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

#[allow(dead_code)]
pub fn create(character: &Character) -> Result<(), Error> {
    println!("Creating character");

    let connection = &mut crate::db::POOL.get()?;

    let _ = diesel::insert_into(schema::characters::table)
        .values(character)
        .execute(connection);

    return Ok(());
}

pub fn delete(character_id: i32, owner_id: u64) -> Result<(), Error> {
    use self::schema::characters::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

    diesel::delete(
        characters
            .filter(id.eq(character_id))
            .filter(user_id.eq(owner_id.to_string())),
    )
    .execute(connection);

    Ok(())
}

pub fn delete_global(character_id: i32) -> Result<QueryResult<usize>, Error> {
    use self::schema::characters::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

    Ok(diesel::delete(characters.filter(id.eq(character_id))).execute(connection))
}

#[allow(dead_code)]
pub fn get_from_user_id(user: u64) -> Result<Vec<Character>, Error> {
    use self::schema::characters::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

    let characters_result = characters
        .filter(user_id.eq(user.to_string()))
        .select(Character::as_select())
        .load(connection)
        .expect("Error loading posts");

    if characters_result.len() > 0 {
        Ok(characters_result)
    } else {
        Err(Box::new(DbError::NotFound))
    }
}

#[allow(dead_code)]
pub fn get_latest(user: u64) -> Result<Character, crate::common::Error> {
    use self::schema::characters::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

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
        Err(Box::new(DbError::NotFound))
    }
}

#[allow(dead_code)]
pub fn get(character_id: i32) -> Result<Character, Error> {
    use self::schema::characters::dsl::*;
    use crate::db::DbError;

    let connection = &mut crate::db::POOL.get()?;

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
        Err(Box::new(DbError::NotFound))
    }
}
#[allow(dead_code)]
pub fn get_by_char_sheet(channel_id: u64, message_id: u64) -> Result<Character, Error> {
    use self::schema::characters::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

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
        Err(Box::new(DbError::NotFound))
    }
}

pub fn update(character: &Character) -> Result<(), Error> {
    use self::schema::characters::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

    let character_id = &character
        .id
        .expect("No character ID provided to update method");

    println!("Updating character {character_id}");

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
