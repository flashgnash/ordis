pub mod models;
pub mod schema;

pub mod characters;

pub mod servers;
pub mod users;

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
