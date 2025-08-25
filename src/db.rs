pub mod models;
pub mod schema;

pub mod characters;

pub mod servers;
pub mod users;

use self::models::*;

use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use diesel::{prelude::*, r2d2};

use std::env;
use std::error::Error;
use std::fmt;

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

lazy_static::lazy_static! {
    pub static ref POOL: Pool = {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let manager = ConnectionManager::<SqliteConnection>::new(db_url);
        Pool::builder().build(manager).unwrap()
    };
}

#[derive(Debug)]
pub enum DbError {
    NotFound,
    DieselError(diesel::result::Error),
    PoolError(r2d2::Error),
}

impl From<diesel::result::Error> for DbError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => DbError::NotFound,
            // only handle specific cases you care about
            _ => DbError::DieselError(err),
        }
    }
}

impl From<diesel::r2d2::Error> for DbError {
    fn from(err: diesel::r2d2::Error) -> Self {
        DbError::PoolError(err)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DbError::NotFound => write!(f, "Not found error"),
            DbError::DieselError(e) => write!(f, "Diesel error: {e}"),
            DbError::PoolError(e) => write!(f, "Connection pool error: {e}"),
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
