// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Text,
        username -> Nullable<Text>,
        count -> Nullable<Integer>,
    }
}
