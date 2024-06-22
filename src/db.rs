 


pub mod models;
pub mod schema;


use self::models::*;


use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;


use std::env;

#[derive(Debug)]
pub enum DbError {
    NotFound,
}

pub fn establish_connection() -> SqliteConnection {

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}



pub struct StatType {
    name: String,
    modifier_action: String
}

pub struct UserStat {
    user_id: u64,
    name: String,
    value: i32,
    stat_block_message: u64,
}




pub fn get_user(user_id: u64) -> Result<User,DbError> {
    use self::schema::users::dsl::*;

    let connection = &mut establish_connection();

    let mut users_result = users
        .filter(id.eq(user_id.to_string()))
        .limit(1)
        .select(User::as_select())
        .load(connection)
        .expect("Error loading posts");

    if users_result.len() > 0 {
        let user = users_result.remove(0);
        Ok(user)
    }
    else {
        Err(DbError::NotFound)
    }
}

