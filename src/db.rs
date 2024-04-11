use sqlite;
use sqlite::State;




pub fn create_database(){
    let connection = sqlite::open("testing.db").unwrap();

    let query = "
        CREATE TABLE users (name TEXT, age INTEGER);
        INSERT INTO users VALUES ('Alice', 42);
        INSERT INTO users VALUES ('Bob', 69);
    ";
    connection.execute(query).unwrap();
   

}

pub fn test(){

    let connection = sqlite::open("testing.db").unwrap();
   
    let query = "SELECT * FROM users;";
    
    let mut statement = connection.prepare(query).unwrap();

    
    while let Ok(State::Row) = statement.next() {
        println!("name = {} age = {}", statement.read::<String, _>("name").unwrap(),statement.read::<i64, _>("age").unwrap());
    }


    //connection.execute(query).unwrap();

}
