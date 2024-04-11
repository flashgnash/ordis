use sqlite;
use sqlite::State;
use std::path::Path;



pub fn get_database() -> sqlite::Connection {
    let dbPath = "testing.db";
    

    if !Path::new(dbPath).exists() {
    
        let connection = sqlite::open(dbPath).unwrap();

        let query = "
            CREATE TABLE users (id INTEGER);
            CREATE TABLE userStats (userId INTEGER, statName TEXT, statValue INTEGER, statAction TEXT);
        ";
        connection.execute(query).unwrap();
        
        return connection;
    }
    else {
        
        return sqlite::open(dbPath).unwrap();

    }

}

pub fn test(){

    let connection = get_database();  

   
    let query = "SELECT * FROM users;";
    
    let mut statement = connection.prepare(query).unwrap();

    
    while let Ok(State::Row) = statement.next() {
        println!("name = {} age = {}", statement.read::<String, _>("name").unwrap(),statement.read::<i64, _>("age").unwrap());
    }


    //connection.execute(query).unwrap();

}
