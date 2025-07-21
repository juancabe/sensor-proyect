use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::sensor_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SensorData {
    pub id: i32,
    pub sensor: String,
    pub serialized_data: String,
    pub added_at: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::sensor_data)]
pub struct NewSensorData<'a> {
    pub sensor: String,
    pub serialized_data: &'a str,
    pub added_at: NaiveDateTime, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::user_places)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserPlace {
    pub id: i32,
    pub user: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_places)]
pub struct NewUserPlace<'a> {
    pub user: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub created_at: NaiveDateTime, // UNIX timestamp in seconds
    pub updated_at: NaiveDateTime, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::user_sensors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSensor {
    pub api_id: String,
    pub place: i32,                      // Foreign key to UserPlace
    pub kind: i32,                       // Foreign key to SensorKind
    pub last_measurement: NaiveDateTime, // UNIX timestamp in seconds
    pub device_id: String,               // 20 bytes HEX String -> Generated at the sensor runtime
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_sensors)]
pub struct NewUserSensor<'a> {
    pub api_id: &'a str, // 20 bytes HEX String -> Generated at the server
    pub place: i32,      // Foreign key to UserPlace
    pub kind: i32,       // Foreign key to SensorKind
    pub last_measurement: NaiveDateTime, // UNIX timestamp in seconds
    pub device_id: &'a str,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
