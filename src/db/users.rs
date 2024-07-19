use models::*;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::db::*;

pub fn create(connection: &mut SqliteConnection, user: &User) -> Result<(), DbError> {
    use self::schema::users::dsl::*;

    println!("Creating user");

    diesel::insert_into(schema::users::table)
        .values(user)
        .execute(connection);

    return Ok(());
}

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

pub fn update(connection: &mut SqliteConnection, user: &User) -> Result<(), DbError> {
    use self::schema::users::dsl::*;

    println!("Updating user");

    let user = diesel::update(users.find(id))
        .set(count.eq(user.count))
        .execute(connection);

    return Ok(());
}
