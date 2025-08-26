// @generated automatically by Diesel CLI.

diesel::table! {
    colors (id) {
        id -> Int4,
        hex_value -> Text,
        name -> Text,
    }
}

diesel::table! {
    sensor_data (id) {
        id -> Int8,
        sensor_id -> Int4,
        data -> Jsonb,
        added_at -> Timestamp,
    }
}

diesel::table! {
    user_places (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        color_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_sensors (id) {
        id -> Int4,
        place_id -> Int4,
        device_id -> Text,
        pub_key -> Text,
        name -> Text,
        description -> Nullable<Text>,
        color_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Text,
        hashed_password -> Text,
        email -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        updated_auth_at -> Timestamp,
    }
}

diesel::joinable!(sensor_data -> user_sensors (sensor_id));
diesel::joinable!(user_places -> colors (color_id));
diesel::joinable!(user_places -> users (user_id));
diesel::joinable!(user_sensors -> colors (color_id));
diesel::joinable!(user_sensors -> user_places (place_id));

diesel::allow_tables_to_appear_in_same_query!(
    colors,
    sensor_data,
    user_places,
    user_sensors,
    users,
);
