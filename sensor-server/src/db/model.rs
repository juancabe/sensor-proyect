use std::array::TryFromSliceError;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use ed25519_dalek::{SignatureError, VerifyingKey};
use hex::FromHexError;

pub type HexValue = String;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::db::schema::colors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Color {
    pub id: i32,
    pub hex_value: HexValue,
    pub name: String,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::db::schema::sensor_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SensorData {
    pub id: i64,
    pub sensor_id: i32,
    pub data: serde_valid::json::Value,
    pub added_at: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::db::schema::sensor_data)]
pub struct NewSensorData {
    pub sensor_id: i32,
    pub data: serde_valid::json::Value,
    pub added_at: Option<NaiveDateTime>, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable, Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::db::schema::user_places)]
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
#[diesel(table_name = crate::db::schema::user_places)]
pub struct NewUserPlace {
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
}

#[derive(Queryable, Selectable, Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::db::schema::user_sensors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSensor {
    pub id: i32,
    pub place_id: i32,     // Foreign key to UserPlace
    pub device_id: String, // 20 bytes HEX String -> Generated at the sensor runtime
    pub pub_key: String,   // ed25519 verify key generated securely in the device, used for
    // sensor data authentication via JWT generation, stored as a HEX encoded string
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::db::schema::user_sensors)]
pub struct NewUserSensor {
    pub place_id: i32, // Foreign key to UserPlace
    pub device_id: String,
    pub_key: String,
    pub name: String,
    pub description: Option<String>,
    pub color_id: i32,
}

#[derive(Debug)]
pub enum InvalidPubKey {
    Hex(FromHexError),
    SizedSlice(TryFromSliceError),
    VerifyingKey(SignatureError),
}

impl NewUserSensor {
    pub fn new(
        place_id: i32,
        device_id: String,
        pub_key: String,
        name: String,
        description: Option<String>,
        color_id: i32,
    ) -> Result<Self, InvalidPubKey> {
        // Validate pub_key
        let pk_bytes = hex::decode(&pub_key).map_err(|e| InvalidPubKey::Hex(e))?;
        // Try to construct it
        VerifyingKey::from_bytes(
            pk_bytes
                .as_slice()
                .try_into()
                .map_err(|e: TryFromSliceError| InvalidPubKey::SizedSlice(e))?,
        )
        .map_err(|e| InvalidPubKey::VerifyingKey(e))?;

        Ok(Self {
            place_id,
            device_id,
            pub_key,
            name,
            description,
            color_id,
        })
    }
}

#[derive(Queryable, Selectable, AsChangeset, Clone, Debug)]
#[diesel(table_name = crate::db::schema::users)]
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
#[diesel(table_name = crate::db::schema::users)]
pub struct NewUser {
    pub username: String,
    pub hashed_password: String,
    pub email: String,
}
