// @generated automatically by Diesel CLI.

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
    user_places (id) {
        id -> Int4,
        user -> Text,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_sensors (api_id) {
        api_id -> Text,
        place -> Int4,
        kind -> Int4,
        last_measurement -> Timestamp,
        device_id -> Text,
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
diesel::joinable!(user_places -> users (user));
diesel::joinable!(user_sensors -> sensor_kinds (kind));
diesel::joinable!(user_sensors -> user_places (place));

diesel::allow_tables_to_appear_in_same_query!(
    sensor_data,
    sensor_kinds,
    user_places,
    user_sensors,
    users,
);
