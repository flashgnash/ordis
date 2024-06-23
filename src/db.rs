pub mod models;
pub mod schema;

use self::models::*;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use std::env;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DbError {
    NotFound,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DbError::NotFound => write!(f, "Not found error"),
            // Other kinds of errors...
        }
    }
}

impl Error for DbError {
    // We don't need to implement anything specifically in here, unless we want a custom cause or backtrace.
}

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub struct StatType {
    name: String,
    modifier_action: String,
}

pub struct UserStat {
    user_id: u64,
    name: String,
    value: i32,
    stat_block_message: u64,
}

pub fn get_user(connection: &mut SqliteConnection, user_id: u64) -> Result<User, DbError> {
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

pub fn update_user(connection: &mut SqliteConnection, user: &User) -> Result<(), DbError> {
    use self::schema::users::dsl::*;

    diesel::insert_into(schema::users::table)
        .values(user)
        .execute(connection);

    return Ok(());
}
