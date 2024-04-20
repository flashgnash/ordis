use rusqlite::{params, Connection, Result};
use rusqlite::types::ToSql;
use rusqlite::Error as SqliteError;  
use std::error::Error; 
use std::fmt;
#[derive(Debug)]
pub enum DbError {                                                                 
        RowNotFound,   
        SqliteError(SqliteError)
}   

impl fmt::Display for DbError {                                                    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {                         
        match *self {                                                              
            DbError::SqliteError(ref e) => write!(f, "SQLite error: {}", e),       
            DbError::RowNotFound => write!(f, "Row not found"),                    
        }                                                                          
    }                                                                              
}    




impl Error for DbError {}  


impl From<SqliteError> for DbError {
    fn from(error: SqliteError) -> Self {
        DbError::SqliteError(error)
    }
}


#[derive(Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub count: Option<i32>
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User {{ id: {}, username: {}, count: {:?} }}", self.id, self.username, self.count)
    }
}

pub struct StatType {
    name: String,
    modifierAction: String
}

pub struct UserStat {
    userId: u64,
    name: String,
    value: i32,
    stat_block_message: u64,
}

pub fn init_database() -> Result<(),rusqlite::Error> {
    
    let connection = Connection::open("testing.db")?;

    let query = "


        CREATE TABLE IF NOT EXISTS users (
            id INTEGER,
            username TEXT,
            count INTEGER DEFAULT 0,

            PRIMARY KEY(id)
        );

        CREATE TABLE IF NOT EXISTS characters (
            name INTEGER NOT NULL,
            userId INTEGER NOT NULL,
            
            PRIMARY KEY(userId,name),
            FOREIGN KEY(userId) REFERENCES users(id)

        );
       
        CREATE TABLE IF NOT EXISTS statTypes (
            name INTEGER NOT NULL, 
            formula TEXT,
            
            PRIMARY KEY(name)
        );

        CREATE TABLE IF NOT EXISTS stats (
            name TEXT NOT NULL,
            value INTEGER NOT NULL,
            character TEXT NOT NULL,

            PRIMARY KEY(character,name),

            FOREIGN KEY (character) REFERENCES characters(userId,name),
            FOREIGN KEY(name) REFERENCES statTypes(name)
        );

    ";


    connection.execute(query,())?;
    
    return Ok(());

}
pub fn setStat(user: User,stat: UserStat) -> Result<(),DbError>{
    
    let connection = Connection::open("testing.db")?;


    return Ok(());
}

pub fn test(user: User) -> Result<User,DbError>{

    init_database()?; 
    let connection = Connection::open("testing.db")?;
   

    let query = "
        INSERT INTO users (id,username)
        VALUES (?1,?2) 
        ON CONFLICT(id) 
        DO UPDATE SET 
            count=count+1,
            username=(?2)
        ;
    ";


    let _ = connection.execute(
        query,
        (&user.id,&user.username)
    )?;

    let mut stmt = connection.prepare("SELECT id,username,count FROM users WHERE id = ?;")?;


    let your_primary_key: &dyn ToSql = &user.id;                                           
    let mut rows = stmt.query_map(params![your_primary_key], |row| {                   
        Ok(User {                                                                
            id: row.get(0)?,                                                           
            username: row.get(1)?,
            count: row.get(2)?
         })                                                                            
    })?;

    if let Some(row) = rows.next() {                                                  
        let user: User = row?;                                            
    } else {                                                                           
        return Err(DbError::RowNotFound); 
    } 


    Ok(user)    


    //connection.execute?(query);

}
