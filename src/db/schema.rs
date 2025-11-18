// @generated automatically by Diesel CLI.

diesel::table! {
    characters (id) {
        id -> Int4,
        user_id -> Nullable<Text>,
        name -> Nullable<Text>,
        stat_block_hash -> Nullable<Text>,
        stat_block -> Nullable<Text>,
        stat_block_message_id -> Nullable<Text>,
        stat_block_channel_id -> Nullable<Text>,
        spell_block_channel_id -> Nullable<Text>,
        spell_block_message_id -> Nullable<Text>,
        spell_block -> Nullable<Text>,
        spell_block_hash -> Nullable<Text>,
        mana -> Nullable<Int4>,
        mana_readout_channel_id -> Nullable<Text>,
        mana_readout_message_id -> Nullable<Text>,
        stat_block_server_id -> Nullable<Text>,
        roll_server_id -> Nullable<Text>,
        saved_rolls -> Nullable<Text>,
    }
}

diesel::table! {
    servers (id) {
        id -> Text,
        default_roll_channel -> Nullable<Text>,
        default_roll_server -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Nullable<Text>,
        count -> Nullable<Int4>,
        stat_block_hash -> Nullable<Text>,
        stat_block -> Nullable<Text>,
        stat_block_message_id -> Nullable<Text>,
        stat_block_channel_id -> Nullable<Text>,
        selected_character_id -> Nullable<Text>,
        selected_character -> Nullable<Int4>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    characters,
    servers,
    users,
);
