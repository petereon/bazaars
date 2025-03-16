// @generated automatically by Diesel CLI.

diesel::table! {
    ads (id) {
        id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        description -> Text,
        price -> Numeric,
        #[max_length = 50]
        status -> Varchar,
        #[max_length = 255]
        user_email -> Varchar,
        #[max_length = 50]
        user_phone -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        top_ad -> Bool,
        images -> Jsonb,
    }
}
