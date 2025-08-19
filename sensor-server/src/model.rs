use chrono::NaiveDateTime;
use diesel::prelude::*;

pub type HexValue = String;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::colors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Color {
    pub id: i32,
    pub hex_value: HexValue,
    pub name: String,
}
pub const COLOR_HEX_STRS: [&'static str; 9] = [
    "#FF0000", "#0000FF", "#FFFF00", "#008000", "#FFA500", "#800080", "#FFFFFF", "#000000",
    "#808080",
];

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::sensor_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SensorData {
    pub id: i64,
    pub sensor_id: i32,
    pub data: serde_json::value::Value,
    pub added_at: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::sensor_data)]
pub struct NewSensorData {
    pub sensor_id: i32,
    pub data: serde_json::value::Value,
    pub added_at: Option<NaiveDateTime>, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_places)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserPlace {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_places)]
pub struct NewUserPlace {
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_sensors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSensor {
    pub id: i32,
    pub place_id: i32,     // Foreign key to UserPlace
    pub device_id: String, // 20 bytes HEX String -> Generated at the sensor runtime
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::user_sensors)]
pub struct NewUserSensor {
    pub place_id: i32, // Foreign key to UserPlace
    pub device_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
}

#[derive(Queryable, Selectable, AsChangeset, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub hashed_password: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub updated_auth_at: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub hashed_password: String,
    pub email: String,
}
