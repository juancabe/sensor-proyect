// @generated automatically by Diesel CLI.

diesel::table! {
    aht10data (data_id) {
        data_id -> Integer,
        user_uuid -> Text,
        user_place_id -> Integer,
        serialized_data -> Text,
        added_at -> BigInt,
    }
}

diesel::table! {
    user_places (user_place_id) {
        user_place_id -> Integer,
        user_id -> Text,
        place_name -> Text,
        place_description -> Nullable<Text>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    users (uuid) {
        uuid -> Text,
        username -> Text,
        hashed_password -> Text,
        email -> Text,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::joinable!(aht10data -> user_places (user_place_id));
diesel::joinable!(aht10data -> users (user_uuid));
diesel::joinable!(user_places -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(aht10data, user_places, users,);
