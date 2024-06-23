use diesel::prelude::*;
use crate::db::schema;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[derive(Insertable)]
#[derive(Clone)]
pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub count: Option<i32>
}
