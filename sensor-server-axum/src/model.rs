use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone)]
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

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::user_places)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserPlace {
    pub id: i32,
    pub api_id: String,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_places)]
pub struct NewUserPlace<'a> {
    pub api_id: &'a str,
    pub user_id: i32,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub color_id: i32,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::user_sensors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSensor {
    pub id: i32,
    pub api_id: String,
    pub place_id: i32,     // Foreign key to UserPlace
    pub device_id: String, // 20 bytes HEX String -> Generated at the sensor runtime
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_sensors)]
pub struct NewUserSensor<'a> {
    pub api_id: &'a str, // 20 bytes HEX String -> Generated at the server
    pub place_id: i32,   // Foreign key to UserPlace
    pub device_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub color_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub api_id: String, // 20 bytes HEX String
    pub hashed_password: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub api_id: &'a str, // e.g 94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA
    pub hashed_password: &'a str,
    pub email: &'a str,
}
