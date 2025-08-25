use crate::common::Error;
use crate::db::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

#[allow(dead_code)]
pub fn create(connection: &mut SqliteConnection, server: &Server) -> Result<(), DbError> {
    println!("Creating server");

    let _ = diesel::insert_into(schema::servers::table)
        .values(server)
        .execute(connection);

    return Ok(());
}

#[allow(dead_code)]
pub fn get(connection: &mut SqliteConnection, server_id: u64) -> Result<Option<Server>, DbError> {
    use self::schema::servers::dsl::*;

    let mut servers_result = servers
        .filter(id.eq(server_id.to_string()))
        .limit(1)
        .select(Server::as_select())
        .load(connection)
        .expect("Error loading posts");

    if servers_result.len() > 0 {
        let server = servers_result.remove(0);
        Ok(Some(server))
    } else {
        Ok(None)
    }
}

pub fn get_or_create(server_id: u64) -> Result<Server, Error> {
    let connection = &mut crate::db::POOL.get()?;
    let server_result = get(connection, server_id);

    match server_result? {
        Some(v) => Ok(v),
        None => {
            let new_server = Server {
                id: server_id.to_string(),
                default_roll_channel: None,
            };
            let _ = servers::create(connection, &new_server);
            Ok(new_server)
        }
    }
}

#[allow(dead_code)]
pub fn update(server: &Server) -> Result<(), Error> {
    use self::schema::servers::dsl::*;
    let connection = &mut crate::db::POOL.get()?;

    let server_id = &server.id.to_string();

    println!("Updating server {server_id}");

    let _ = diesel::update(servers.filter(id.eq(server_id)))
        .set(server)
        .execute(connection);

    return Ok(());
}
