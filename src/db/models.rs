use crate::db::schema;
use diesel::prelude::*;

#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[derive(Insertable, Clone)]
pub struct User {
    pub id: String,
    pub username: Option<String>,
    pub count: Option<i32>,
    pub selected_character: Option<i32>,
}
impl Character {
    pub fn new_empty() -> Character {
        Character {
            id: 0,
            user_id: None,
            name: None,

            roll_server_id: None,

            stat_block_hash: None,
            stat_block: None,
            stat_block_message_id: None,
            stat_block_channel_id: None,

            spell_block_channel_id: None,
            spell_block_message_id: None,
            spell_block: None,
            spell_block_hash: None,

            mana: None,
            mana_readout_channel_id: None,
            mana_readout_message_id: None,

            saved_rolls: None,
            stat_block_server_id: None,

            campaign_id: None,
        }
    }
}

#[derive(Queryable, Selectable, AsChangeset, Debug, Clone)]
#[diesel(table_name = schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User))]
#[derive(serde::Deserialize)]
pub struct Character {
    pub id: i32,
    pub user_id: Option<String>,
    pub name: Option<String>,

    pub roll_server_id: Option<String>,

    pub stat_block_hash: Option<String>,
    pub stat_block: Option<String>,
    pub stat_block_message_id: Option<String>,
    pub stat_block_channel_id: Option<String>,

    pub spell_block_channel_id: Option<String>,
    pub spell_block_message_id: Option<String>,
    pub spell_block: Option<String>,
    pub spell_block_hash: Option<String>,

    pub mana: Option<i32>,
    pub mana_readout_channel_id: Option<String>,
    pub mana_readout_message_id: Option<String>,

    pub saved_rolls: Option<String>,
    pub stat_block_server_id: Option<String>,

    #[diesel(column_name = CampaignId)]
    pub campaign_id: Option<i32>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = schema::characters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewCharacter {
    pub user_id: Option<String>,
    pub name: Option<String>,
    pub roll_server_id: Option<String>,
    pub stat_block_hash: Option<String>,
    pub stat_block: Option<String>,
    pub stat_block_message_id: Option<String>,
    pub stat_block_channel_id: Option<String>,
    pub spell_block_channel_id: Option<String>,
    pub spell_block_message_id: Option<String>,
    pub spell_block: Option<String>,
    pub spell_block_hash: Option<String>,
    pub mana: Option<i32>,
    pub mana_readout_channel_id: Option<String>,
    pub mana_readout_message_id: Option<String>,
    pub saved_rolls: Option<String>,
    pub stat_block_server_id: Option<String>,
    #[diesel(column_name = CampaignId)]
    pub campaign_id: Option<i32>,
}

impl From<&Character> for NewCharacter {
    fn from(character: &Character) -> Self {
        NewCharacter {
            user_id: character.user_id.clone(),
            name: character.name.clone(),
            roll_server_id: character.roll_server_id.clone(),
            stat_block_hash: character.stat_block_hash.clone(),
            stat_block: character.stat_block.clone(),
            stat_block_message_id: character.stat_block_message_id.clone(),
            stat_block_channel_id: character.stat_block_channel_id.clone(),
            spell_block_channel_id: character.spell_block_channel_id.clone(),
            spell_block_message_id: character.spell_block_message_id.clone(),
            spell_block: character.spell_block.clone(),
            spell_block_hash: character.spell_block_hash.clone(),
            mana: character.mana,
            mana_readout_channel_id: character.mana_readout_channel_id.clone(),
            mana_readout_message_id: character.mana_readout_message_id.clone(),
            saved_rolls: character.saved_rolls.clone(),
            stat_block_server_id: character.stat_block_server_id.clone(),
            campaign_id: character.campaign_id,
        }
    }
}

#[derive(Queryable, Selectable, AsChangeset, Debug)]
#[diesel(table_name = schema::servers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[derive(Insertable, Clone)]
pub struct Server {
    pub id: String,
    pub default_roll_channel: Option<String>,
    pub default_roll_server: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = schema::Gauges)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Character, foreign_key = PlayerCharacterId))]
pub struct Gauge {
    #[diesel(column_name = Id)]
    pub id: uuid::Uuid,
    #[diesel(column_name = Icon)]
    pub icon: Option<String>,
    #[diesel(column_name = Name)]
    pub name: String,
    #[diesel(column_name = Value)]
    pub value: i32,
    #[diesel(column_name = Max)]
    pub max: i32,
    #[diesel(column_name = PlayerCharacterId)]
    pub player_character_id: i32,
    #[diesel(column_name = GaugeType)]
    pub gauge_type: i32,
    #[diesel(column_name = Colour)]
    pub colour: Option<String>,
}
