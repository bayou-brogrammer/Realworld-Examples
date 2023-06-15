// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        #[max_length = 254]
        email -> Varchar,
        password -> Text,
        bio -> Nullable<Text>,
        image -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
