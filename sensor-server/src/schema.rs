// @generated automatically by Diesel CLI.

diesel::table! {
    aht10data (id) {
        id -> Integer,
        sensor -> Text,
        serialized_data -> Text,
        added_at -> Integer,
    }
}

diesel::table! {
    scd4xdata (id) {
        id -> Integer,
        sensor -> Text,
        serialized_data -> Text,
        added_at -> Integer,
    }
}

diesel::table! {
    sensor_kinds (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    user_places (id) {
        id -> Integer,
        user -> Text,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    user_sensors (api_id) {
        api_id -> Text,
        place -> Integer,
        kind -> Integer,
        last_measurement -> Integer,
        ble_mac_address -> Text,
    }
}

diesel::table! {
    users (uuid) {
        uuid -> Text,
        username -> Text,
        hashed_password -> Text,
        email -> Text,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::joinable!(aht10data -> user_sensors (sensor));
diesel::joinable!(scd4xdata -> user_sensors (sensor));
diesel::joinable!(user_places -> users (user));
diesel::joinable!(user_sensors -> sensor_kinds (kind));
diesel::joinable!(user_sensors -> user_places (place));

diesel::allow_tables_to_appear_in_same_query!(
    aht10data,
    scd4xdata,
    sensor_kinds,
    user_places,
    user_sensors,
    users,
);
