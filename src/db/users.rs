use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::db::*;

#[allow(dead_code)]
pub fn create(connection: &mut SqliteConnection, user: &User) -> Result<(), DbError> {
    println!("Creating user");

    let _ = diesel::insert_into(schema::users::table)
        .values(user)
        .execute(connection);

    return Ok(());
}

#[allow(dead_code)]
pub fn get(connection: &mut SqliteConnection, user_id: u64) -> Result<User, DbError> {
    use self::schema::users::dsl::*;

    let mut users_result = users
        .filter(id.eq(user_id.to_string()))
        .limit(1)
        .select(User::as_select())
        .load(connection)
        .expect("Error loading posts");

    if users_result.len() > 0 {
        let user = users_result.remove(0);
        Ok(user)
    } else {
        Err(DbError::NotFound)
    }
}

pub fn get_or_create(connection: &mut SqliteConnection, user_id: u64) -> Result<User, DbError> {
    let user_result = get(connection, user_id);

    match user_result {
        Ok(v) => Ok(v),
        Err(e) => {
            //This is probably not a good plan; catches all error types
            //TODO find a way to catch specific exceptions
            println!("User not found ({}). Creating a new one", e);

            let new_user = User {
                id: user_id.to_string(),
                username: None,
                count: Some(1),
                stat_block: None,
                stat_block_hash: None,

                stat_block_message_id: None,
                stat_block_channel_id: None,
            };
            let _ = users::create(connection, &new_user);
            Ok(new_user)
        }
    }
}

#[allow(dead_code)]
pub fn update(connection: &mut SqliteConnection, user: &User) -> Result<(), DbError> {
    use self::schema::users::dsl::*;

    let user_id = &user.id.to_string();

    println!("Updating user {user_id}");

    let _ = diesel::update(users.filter(id.eq(user_id)))
        .set(user)
        .execute(connection);

    return Ok(());
}
