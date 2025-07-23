// @generated automatically by Diesel CLI.

diesel::table! {
    place_color (hex_value) {
        hex_value -> Text,
    }
}

diesel::table! {
    sensor_color (hex_value) {
        hex_value -> Text,
    }
}

diesel::table! {
    sensor_data (id) {
        id -> Int4,
        sensor -> Text,
        serialized_data -> Text,
        added_at -> Timestamp,
    }
}

diesel::table! {
    sensor_kinds (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    user_places (api_id) {
        api_id -> Text,
        user -> Text,
        name -> Text,
        description -> Nullable<Text>,
        color -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_sensors (api_id) {
        api_id -> Text,
        place -> Text,
        kind -> Int4,
        last_measurement -> Timestamp,
        device_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        color -> Text,
    }
}

diesel::table! {
    users (username) {
        username -> Text,
        api_id -> Text,
        hashed_password -> Text,
        email -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(sensor_data -> user_sensors (sensor));
diesel::joinable!(user_places -> place_color (color));
diesel::joinable!(user_places -> users (user));
diesel::joinable!(user_sensors -> sensor_color (color));
diesel::joinable!(user_sensors -> sensor_kinds (kind));
diesel::joinable!(user_sensors -> user_places (place));

diesel::allow_tables_to_appear_in_same_query!(
    place_color,
    sensor_color,
    sensor_data,
    sensor_kinds,
    user_places,
    user_sensors,
    users,
);
