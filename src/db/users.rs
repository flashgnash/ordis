use crate::common::Error;
use crate::db::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

#[allow(dead_code)]
pub fn create(connection: &mut SqliteConnection, user: &User) -> Result<(), DbError> {
    println!("Creating user");

    let _ = diesel::insert_into(schema::users::table)
        .values(user)
        .execute(connection);

    return Ok(());
}

#[allow(dead_code)]
pub fn get(user_id: u64) -> Result<User, Error> {
    use self::schema::users::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

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
        Err(Box::new(DbError::NotFound))
    }
}

pub fn get_or_create(user_id: u64) -> Result<User, Error> {
    let connection = &mut crate::db::POOL.get()?;

    let user_result = get(user_id);

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
                selected_character: None,
            };
            let _ = users::create(connection, &new_user);
            Ok(new_user)
        }
    }
}

#[allow(dead_code)]
pub fn update(user: &User) -> Result<(), Error> {
    use self::schema::users::dsl::*;

    let connection = &mut crate::db::POOL.get()?;

    let user_id = &user.id.to_string();

    println!("Updating user {user_id}");

    let _ = diesel::update(users.filter(id.eq(user_id)))
        .set(user)
        .execute(connection);

    return Ok(());
}

#[allow(dead_code)]
pub fn unset_character(user: &User) -> Result<(), Error> {
    use self::schema::users::dsl::*;
    let connection = &mut crate::db::POOL.get()?;

    let user_id = &user.id.to_string();

    println!("Removing selected character for user {user_id}");

    let null_value: Option<i32> = None;

    let _ = diesel::update(users.filter(id.eq(user_id)))
        .set((selected_character.eq(null_value),))
        .execute(connection);

    return Ok(());
}
