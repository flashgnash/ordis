// @generated automatically by Diesel CLI.

diesel::table! {
    characters (id) {
        id -> Nullable<Integer>,
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
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Nullable<Text>,
        count -> Nullable<Integer>,
        stat_block_hash -> Nullable<Text>,
        stat_block -> Nullable<Text>,
        stat_block_message_id -> Nullable<Text>,
        stat_block_channel_id -> Nullable<Text>,
        selected_character_id -> Nullable<Text>,
        selected_character -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    characters,
    users,
);
