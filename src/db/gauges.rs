use crate::common::Error;
use super::models::Gauge;
use super::schema::Gauges::dsl::*;
use super::POOL;
use diesel::prelude::*;

pub fn get_for_character(character_id: i32) -> Result<Vec<Gauge>, Error> {
    let mut connection = POOL.get()?;

    let results = Gauges
        .filter(PlayerCharacterId.eq(character_id))
        .select(Gauge::as_select())
        .load(&mut connection)?;

    Ok(results)
}
