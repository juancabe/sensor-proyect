use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::aht10data)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Aht10Data {
    pub id: i32,
    pub sensor: String,
    pub serialized_data: String,
    pub added_at: i32,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::aht10data)]
pub struct NewAht10Data<'a> {
    pub sensor: String,
    pub serialized_data: &'a str,
    pub added_at: i32, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::scd4xdata)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Scd4xData {
    pub id: i32,
    pub sensor: String,
    pub serialized_data: String,
    pub added_at: i32,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::scd4xdata)]
pub struct NewScd4xData<'a> {
    pub sensor: String,
    pub serialized_data: &'a str,
    pub added_at: i32, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_places)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserPlace {
    pub id: i32,
    pub user: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_places)]
pub struct NewUserPlace<'a> {
    pub user: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub created_at: i32, // UNIX timestamp in seconds
    pub updated_at: i32, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_sensors)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserSensor {
    pub api_id: String,
    pub place: i32,            // Foreign key to UserPlace
    pub kind: i32,             // Foreign key to SensorKind
    pub last_measurement: i32, // UNIX timestamp in seconds
    pub ble_mac_address: String,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_sensors)]
pub struct NewUserSensor<'a> {
    pub api_id: &'a str,          // Unique identifier for the user sensor API
    pub place: i32,               // Foreign key to UserPlace
    pub kind: i32,                // Foreign key to SensorKind
    pub last_measurement: i32,    // UNIX timestamp in seconds
    pub ble_mac_address: &'a str, // BLE MAC address of the sensor
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub uuid: String,
    pub username: String,
    pub hashed_password: String,
    pub email: String,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub uuid: &'a str, // UUID of the user
    pub username: &'a str,
    pub hashed_password: &'a str,
    pub email: &'a str,
    pub created_at: i32,
    pub updated_at: i32,
}
