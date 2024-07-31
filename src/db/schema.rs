// @generated automatically by Diesel CLI.

diesel::table! {
    characters (id, user_id) {
        id -> Text,
        user_id -> Text,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Nullable<Text>,
        count -> Nullable<Integer>,
        stat_block_hash -> Nullable<Text>,
        stat_block -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    characters,
    users,
);
