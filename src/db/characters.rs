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

#[allow(dead_code)]
pub fn get(connection: &mut SqliteConnection, character_id: u64) -> Result<Character, DbError> {
    use self::schema::characters::dsl::*;

    let mut characters_result = characters
        .filter(id.eq(character_id.to_string()))
        .limit(1)
        .select(Character::as_select())
        .load(connection)
        .expect("Error loading posts");

    if characters_result.len() > 0 {
        let character = characters_result.remove(0);
        Ok(character)
    } else {
        Err(DbError::NotFound)
    }
}

// #[allow(dead_code)]
// pub fn update(connection: &mut SqliteConnection, character: &Character) -> Result<(), DbError> {
//     println!("Updating character");

//     let character = diesel::update(characters.find(id))
//         .set(count.eq(character.count))
//         .execute(connection);

//     return Ok(());
// }
