use crate::db::schema;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[derive(Insertable, Clone)]
pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub count: Option<i32>,
    pub stat_block: Option<String>,
    pub stat_block_hash: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::characters)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[derive(Insertable, Clone)]
#[diesel(belongs_to(User))]
pub struct Character {
    pub id: String,
    pub user_id: String,
    pub name: Option<String>,
}
