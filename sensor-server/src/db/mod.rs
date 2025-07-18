use chrono::NaiveDateTime;
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;

use once_cell::sync::Lazy;
use sensor_lib::api;
use sensor_lib::api::endpoints::get_sensor_data::GetSensorDataRequestBody;
use sensor_lib::api::endpoints::register::RegisterRequestBody;
use sensor_lib::api::model::api_id::ApiId;
use sensor_lib::api::model::sensor_kind::{SensorKind, SensorKindData};

use crate::models::{self, NewUser};

use r2d2::Error as DieselPoolError;

type FailedDeserialize = usize;

#[derive(Debug)]
pub enum Error {
    DataBaseError(diesel::result::Error),
    DataBaseConnectionError,
    SerializationError(serde_json::Error),
    DeviceIdInvalid,
    Inconsistency(String),
    NotFound,
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::DataBaseError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err)
    }
}

impl From<DieselPoolError> for crate::db::Error {
    fn from(_: DieselPoolError) -> Self {
        Error::DataBaseConnectionError
    }
}

type DbPool = r2d2::Pool<ConnectionManager<diesel::PgConnection>>;

static DB_POOL: Lazy<DbPool> = Lazy::new(|| {
    dotenv().expect("Failed to read .env file");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<diesel::PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
});

pub fn get_db_pool()
-> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, crate::db::Error> {
    let db: r2d2::PooledConnection<ConnectionManager<PgConnection>> = DB_POOL
        .get()
        .map_err(|_| crate::db::Error::DataBaseConnectionError)?;

    Ok(db)
}

pub fn test_db_pool() -> Result<(), ()> {
    DB_POOL.get().map_err(|_| ())?;
    log::info!("Test database connection established successfully");
    Ok(())
}

pub fn query_sensor_data(
    query: GetSensorDataRequestBody,
) -> Result<(Vec<SensorKindData>, FailedDeserialize), Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let r = users_table
        .filter(users::api_id.eq(&query.user_api_id.as_str()))
        .inner_join(user_places_table)
        .inner_join(user_sensors_table.on(user_places::id.eq(user_sensors::place)))
        .filter(user_sensors::api_id.eq(&query.sensor_api_id.as_str()))
        .select(models::UserSensor::as_select())
        .load::<models::UserSensor>(&mut db_conn)?;

    match r.first() {
        Some(s) => match SensorKind::from_i32(s.kind) {
            Some(k) => match k {
                SensorKind::Aht10 => Ok(query_aht10_data(query)?),
                SensorKind::Scd4x => Ok(query_scd4x_data(query)?),
            },
            None => Err(Error::Inconsistency(String::from(
                "SensorKind in DB is wrong",
            )))?,
        },
        None => Err(Error::NotFound)?,
    }
}

pub fn query_aht10_data(
    query: GetSensorDataRequestBody,
) -> Result<(Vec<SensorKindData>, FailedDeserialize), Error> {
    use crate::schema::{
        aht10data::dsl as aht10data, aht10data::dsl::aht10data as aht10data_table,
    };
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let max_date = query.added_at_upper;
    let min_date = query.added_at_lower;

    let mut failed_deserialize: usize = 0;

    let vec = aht10data_table
        .filter(aht10data::sensor.eq(query.sensor_api_id.as_str()))
        .inner_join(user_sensors_table)
        .inner_join(user_places_table.on(user_places::id.eq(user_sensors::place)))
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(query.user_api_id.as_str()))
        .filter(aht10data::added_at.le(max_date.unwrap_or(NaiveDateTime::MAX)))
        .filter(aht10data::added_at.ge(min_date.unwrap_or(NaiveDateTime::from_timestamp_opt(0, 0).expect("(secs: 0, nsecs: 0) should be valid timestamp"))))
        .select(models::Aht10Data::as_select())
        .load::<models::Aht10Data>(&mut db_conn)?
        .into_iter()
        .map(|data| {
            let deserialized = serde_json::from_str::<api::model::aht10_data::Aht10Data>(&data.serialized_data);

            match deserialized {
                Ok(value) => Some(SensorKindData::Aht10(value)),
                Err(err) => {
                    failed_deserialize += 1;
                    log::error!(
                        "INCONSISTENCY -> Failed to deserialize previously serialized AHT10 data: {}",
                        err
                    );
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect();

    Ok((vec, failed_deserialize))
}

pub fn query_scd4x_data(
    query: GetSensorDataRequestBody,
) -> Result<(Vec<SensorKindData>, FailedDeserialize), Error> {
    use crate::schema::{
        scd4xdata::dsl as scd4xdata, scd4xdata::dsl::scd4xdata as scd4xdata_table,
    };
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let max_date = query.added_at_upper;
    let min_date = query.added_at_lower;

    let mut failed_deserialize: usize = 0;

    let vec = scd4xdata_table
        .filter(scd4xdata::sensor.eq(query.sensor_api_id.as_str()))
        .inner_join(user_sensors_table)
        .inner_join(user_places_table.on(user_places::id.eq(user_sensors::place)))
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(query.user_api_id.as_str()))
        .filter(scd4xdata::added_at.le(max_date.unwrap_or(NaiveDateTime::MAX)))
        .filter(scd4xdata::added_at.ge(min_date.unwrap_or(NaiveDateTime::from_timestamp_opt(0, 0).expect("(secs: 0, nsecs: 0) should be valid timestamp"))))
        .select(models::Scd4xData::as_select())
        .load::<models::Scd4xData>(&mut db_conn)?
        .into_iter()
        .map(|data| {
            let deserialized =
                serde_json::from_str::<api::model::scd4x_data::Scd4xData>(&data.serialized_data);

            match deserialized {
                Ok(value) => Some(SensorKindData::Scd4x(value)),
                Err(err) => {
                    failed_deserialize += 1;
                    log::error!(
                        "INCONSISTENCY -> Failed to deserialize previously serialized SCD4X data: {}",
                        err
                    );
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect();

    Ok((vec, failed_deserialize))
}

pub fn get_user_from_sensor(sensor_api_id: &str) -> Result<String, crate::db::Error> {
    // use crate::schema::user_places::dsl::*;
    // use crate::schema::user_sensors::dsl::*;

    let sensor_api_id = sensor_api_id;

    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };

    let mut db_conn = get_db_pool()?;

    let user_id_result = user_sensors_table
        .inner_join(user_places_table)
        .filter(user_sensors::api_id.eq(&sensor_api_id))
        .select(user_places::user)
        .first::<String>(&mut db_conn);

    match user_id_result {
        Ok(uid) => Ok(uid),
        Err(err) => {
            log::warn!(
                "Failed to find user for sensor with API ID {}: {:?}",
                sensor_api_id,
                err
            );
            Err(crate::db::Error::DataBaseError(err))
        }
    }
}

pub fn user_api_id_matches_sensor_api_id(
    user_api_id: &str,
    sensor_api_id: &str,
) -> Result<bool, Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let count = user_sensors_table
        .inner_join(user_places_table.on(user_places::id.eq(user_sensors::place)))
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(user_api_id))
        .filter(user_sensors::api_id.eq(sensor_api_id))
        .select(count_star())
        .first::<i64>(&mut db_conn)?;

    Ok(count > 0)
}

pub fn user_api_id_matches_place_id(
    user_api_id: &str,
    user_place_id: i32,
) -> Result<bool, crate::db::Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let count = user_places_table
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(user_api_id))
        .filter(user_places::id.eq(user_place_id))
        .select(count_star())
        .first::<i64>(&mut db_conn)?;

    Ok(count > 0)
}

pub fn get_sensor_kind_from_id(sensor_api_id: &str) -> Result<SensorKind, crate::db::Error> {
    // use crate::schema::user_sensors::dsl::*;
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };

    let mut db_conn = get_db_pool()?;

    let kind = user_sensors_table
        .filter(user_sensors::api_id.eq(sensor_api_id))
        .select(user_sensors::kind)
        .first::<i32>(&mut db_conn)?;

    Ok(SensorKind::from_u8(kind as u8)
        .ok_or_else(|| crate::db::Error::DataBaseError(diesel::result::Error::NotFound))?)
}

pub fn save_new_aht10_data(data: models::NewAht10Data<'_>) -> Result<(), Error> {
    use crate::schema::aht10data;

    log::info!("Saving new AHT10 data: {:?}", data);

    let mut db_conn: r2d2::PooledConnection<ConnectionManager<PgConnection>> = get_db_pool()?;

    diesel::insert_into(aht10data::table)
        .values(data)
        .execute(&mut db_conn)?;
    Ok(())
}

pub fn save_new_scd4x_data(data: models::NewScd4xData<'_>) -> Result<(), Error> {
    use crate::schema::scd4xdata;

    log::info!("Saving new SCD4X data: {:?}", data);

    let mut db_conn: r2d2::PooledConnection<ConnectionManager<PgConnection>> = get_db_pool()?;

    diesel::insert_into(scd4xdata::table)
        .values(data)
        .execute(&mut db_conn)?;
    Ok(())
}

pub fn generate_sensor_api_id(
    user_uuid: &str,
    sensor_kind: &SensorKind,
    user_place_id: i32,
    device_id: &str,
) -> Result<[u8; 20], Error> {
    let device_id = device_id;

    // Check device_id format
    if device_id.len() != 40
        || device_id
            .chars()
            .into_iter()
            .any(|c| !c.is_ascii_hexdigit())
    {
        return Err(Error::DeviceIdInvalid);
    }

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(user_uuid.as_bytes());
    hasher.update(sensor_kind.as_str().as_bytes());
    hasher.update(user_place_id.to_string().as_bytes());
    hasher.update(device_id.as_bytes());
    let hash = hasher.finalize();
    let mut sensor_id = [0u8; 20];
    sensor_id.copy_from_slice(&hash[..20]);
    Ok(sensor_id)
}

// Returns sensor api id if already present on DB
pub fn sensor_exists(
    user_api_id: &str,
    user_place_id: i32,
    device_id: &str,
) -> Result<Option<String>, Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let res = users_table
        .filter(users::api_id.eq(user_api_id))
        .inner_join(user_places_table)
        .filter(user_places::id.eq(user_place_id))
        .inner_join(user_sensors_table.on(user_sensors::device_id.eq(device_id)))
        .select(user_sensors::api_id)
        .first::<String>(&mut db_conn);

    match res {
        Ok(r) => Ok(Some(r)),
        Err(e) => match e {
            diesel::result::Error::NotFound => Ok(None),
            e => Err(Error::DataBaseError(e)),
        },
    }
}

pub fn new_sensor(
    user_api_id: &str,
    sensor_kind: SensorKind,
    user_place_id: i32,
    device_id: &str,
) -> Result<String, Error> {
    use crate::schema::user_sensors;

    let user_api_id = user_api_id;

    if !user_api_id_matches_place_id(&user_api_id, user_place_id)? {
        return Err(Error::DataBaseError(diesel::result::Error::NotFound));
    }

    let mut db_conn = get_db_pool()?;

    let user_sensor_api_id =
        generate_sensor_api_id(&user_api_id, &sensor_kind, user_place_id, device_id);
    let user_sensor_api_id = hex::encode(user_sensor_api_id?);

    let new_sensor = models::NewUserSensor {
        api_id: &user_sensor_api_id,
        place: user_place_id,
        kind: sensor_kind as i32,
        last_measurement: NaiveDateTime::from_timestamp_opt(0, 0)
            .expect("(secs: 0, nsecs: 0) should be valid timestamp"),
        device_id,
    };

    diesel::insert_into(user_sensors::table)
        .values(new_sensor)
        .execute(&mut db_conn)?;

    Ok(user_sensor_api_id)
}

pub fn get_login(username: &str, hashed_password: &str) -> Result<Option<String>, Error> {
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};
    let mut db_conn = get_db_pool()?;
    let user_api_id = users_table
        .filter(users::username.eq(username))
        .filter(users::hashed_password.eq(hashed_password))
        .select(users::api_id)
        .first::<String>(&mut db_conn);

    match user_api_id {
        Ok(user_api_id) => Ok(Some(user_api_id)),
        Err(e) => match e {
            diesel::result::Error::NotFound => Ok(None),
            _ => Err(Error::DataBaseError(e)),
        },
    }
}

#[derive(Debug)]
pub enum NewUserError {
    EmailUsed,
    UsernameUsed,
    OtherError(Error),
}

pub fn new_user(query: RegisterRequestBody) -> Result<ApiId, NewUserError> {
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool().map_err(NewUserError::OtherError)?;

    // Check if email is already used
    match users_table
        .filter(users::email.eq(&query.email))
        .select(users::username)
        .first::<String>(&mut db_conn)
    {
        Ok(_) => return Err(NewUserError::EmailUsed),
        Err(e) => match e {
            diesel::result::Error::NotFound => (),
            _ => return Err(NewUserError::OtherError(Error::DataBaseError(e))),
        },
    }

    match users_table
        .filter(users::username.eq(&query.username))
        .select(users::username)
        .first::<String>(&mut db_conn)
    {
        Ok(_) => return Err(NewUserError::UsernameUsed),
        Err(e) => match e {
            diesel::result::Error::NotFound => (),
            _ => return Err(NewUserError::OtherError(Error::DataBaseError(e))),
        },
    }

    // Create new user
    let api_id = ApiId::new();
    let new_user = models::NewUser {
        api_id: &api_id.to_string(),
        username: &query.username,
        email: &query.email,
        hashed_password: &query.hashed_password,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(users_table)
        .values(new_user)
        .execute(&mut db_conn)
        .map_err(|e| NewUserError::OtherError(Error::DataBaseError(e)))?;

    Ok(api_id)
}

// Returns NONE if user does not exist, Some(()) if user was deleted successfully
pub fn delete_user(api_id: &str) -> Result<Option<()>, Error> {
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    match diesel::delete(users_table.filter(users::api_id.eq(api_id))).execute(&mut db_conn) {
        Ok(_) => Ok(Some(())),
        Err(e) => match e {
            diesel::result::Error::NotFound => Ok(None),
            _ => Err(Error::DataBaseError(e)),
        },
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_new_user_then_delete() {
        let query = RegisterRequestBody {
            username: "testuser_not_exists".to_string(),
            hashed_password: "hashedpassword123".to_string(), // Replace with actual hashed password
            email: "testuser_not_exists@example.com".to_string(),
        };

        let r = new_user(query);
        let user_api_id = r.expect("Should be able to create new user with DB");

        // Delete
        match delete_user(&user_api_id.to_string()) {
            Ok(opt) => assert!(
                opt.is_some(),
                "User should be deleted successfully, it isn't"
            ),
            Err(_) => panic!("Should be able to delete user with DB"),
        }
    }

    #[test]
    fn test_get_login() {
        let username = "testuser";
        let hashed_password = "hashedpassword123"; // Replace with actual hashed password

        let result = get_login(username, hashed_password);
        assert!(result.is_ok(), "Failed to get login: {:?}", result.err());
        let user_uuid = result.unwrap().expect("Should be Some");
        assert!(!user_uuid.is_empty(), "User UUID should not be empty");
    }

    #[test]
    fn test_get_login_nonexistent() {
        let username = "nonexistentuser";
        let hashed_password = "hashedpassword123"; // Replace with actual hashed password

        let result = get_login(username, hashed_password);
        assert!(
            result.is_ok_and(|none| none.is_none()),
            "Expected None when getting login with incorrect password"
        );
    }

    #[test]
    fn test_get_login_incorrect_password() {
        let username = "testuser";
        let hashed_password = "wronghashedpassword"; // Replace with actual hashed password

        let result = get_login(username, hashed_password);
        assert!(
            result.is_ok_and(|none| none.is_none()),
            "Expected None when getting login with incorrect password"
        );
    }

    #[test]
    fn test_generate_sensor_id() {
        let user_uuid = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let sensor_kind = SensorKind::Aht10;
        let user_place_id = 1; // Assuming this is a valid place ID for the test user
        let device_ids_ok = [
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaab",
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaac",
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaad",
        ];
        let device_ids_invalid = [
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaabhola", // Too long invalid
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaahola",     // Invalid chars
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaa",         // Too short
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaabaaaa", // Too long valid
        ];

        for device_id in device_ids_ok {
            let sensor_api_id =
                generate_sensor_api_id(user_uuid, &sensor_kind, user_place_id, device_id)
                    .expect("Failed to generate sensor ID");
            assert_eq!(
                sensor_api_id.len(),
                20,
                "Sensor ID should be 20 bytes long, got {} for {:?}",
                sensor_api_id.len(),
                sensor_api_id
            );
        }

        for device_id in device_ids_invalid {
            let result = generate_sensor_api_id(user_uuid, &sensor_kind, user_place_id, device_id);
            assert!(
                result.is_err(),
                "Expected error for invalid device_id: {}",
                device_id
            );
            if let Err(Error::DeviceIdInvalid) = result {
                // Expected error for invalid device_id
            } else {
                panic!("Expected a DeviceIdInvalid error, but got: {:?}", result);
            }
        }
    }

    #[test]
    fn test_new_sensor_then_delete() {
        let user_uuid = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let sensor_kind = SensorKind::Aht10;
        let user_place_id = 1; // Assuming this is a valid place ID for the test user
        let device_id = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaab";

        let result = new_sensor(user_uuid, sensor_kind, user_place_id, device_id)
            .expect("Should be able to create new sensor with DB");

        assert!(!result.is_empty(), "Sensor API ID should not be empty");

        let errored_result = new_sensor(user_uuid, sensor_kind, user_place_id, device_id);
        assert!(
            errored_result.is_err(),
            "Expected error when creating sensor with existing API ID"
        );

        let errored_result = new_sensor(
            "nonexistent-user-uuid",
            sensor_kind,
            user_place_id,
            device_id,
        );
        assert!(
            errored_result.is_err(),
            "Expected error when creating sensor for nonexistent user"
        );

        let errored_result = new_sensor(
            user_uuid,
            sensor_kind, // Assuming this is an invalid sensor kind
            123,         // Assuming this is an invalid place ID
            device_id,
        );
        assert!(
            errored_result.is_err(),
            "Expected error when creating sensor with invalid place ID"
        );

        // Clean up by deleting the sensor
        let mut db_conn = get_db_pool().expect("Failed to get database connection");

        use crate::schema::{
            user_sensors::dsl as user_sensors,
            user_sensors::dsl::user_sensors as user_sensors_table,
        };

        diesel::delete(user_sensors_table.filter(user_sensors::api_id.eq(&result)))
            .execute(&mut db_conn)
            .expect("Failed to delete sensor");
    }

    #[test]
    fn test_user_api_id_matches_sensor_api_id() {
        let user_api_id = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let sensor_api_id = "abc36768cf4d927e267a72ac1cb8108693bdafd1";
        let matches = user_api_id_matches_sensor_api_id(user_api_id, sensor_api_id)
            .expect("Failed to check if user UUID matches sensor API ID");
        assert!(
            matches,
            "Expected user UUID to match sensor API ID, but it did not"
        );
    }

    #[test]
    fn test_user_uuid_matches_sensor_api_id_nonexistent() {
        let user_uuid = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let sensor_api_id = "nonexistent-sensor";
        let result = user_api_id_matches_sensor_api_id(user_uuid, sensor_api_id)
            .expect("Failed to check if user UUID matches nonexistent sensor API ID");
        assert!(
            !result,
            "Expected user UUID to not match nonexistent sensor API ID, but it did"
        );
    }

    #[test]
    fn test_user_uuid_matches_place_id() {
        let user_uuid = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let user_place_id = 1; // Assuming this is a valid place ID for the test user
        let result = user_api_id_matches_place_id(user_uuid, user_place_id);
        assert!(
            result.is_ok(),
            "Failed to check if user UUID matches place ID: {:?}",
            result.err()
        );
        assert!(
            result.unwrap(),
            "Expected user UUID to match place ID, but it did not"
        );
    }

    #[test]
    fn test_user_uuid_matches_place_id_nonexistent() {
        let user_uuid = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let user_place_id = 999; // Assuming this is an invalid place ID
        let result = user_api_id_matches_place_id(user_uuid, user_place_id).expect("no error");
        assert!(
            !result,
            "Expected user UUID to not match nonexistent place ID, but it did"
        );
    }

    #[test]
    fn test_get_user_from_sensor() {
        let sensor_api_id = "94a990533d761111111111111111111111111111";
        let result = get_user_from_sensor(sensor_api_id);
        assert!(
            result.is_ok(),
            "Failed to get user from sensor: {:?}",
            result.err()
        );
        let username = result.unwrap();
        assert!(!username.is_empty(), "username should not be empty");
        assert!(
            username.eq("testuser"),
            "Expected username to match testuser"
        );
    }

    #[test]
    fn test_get_user_from_sensor_nonexistent() {
        let sensor_api_id = "nonexistent-sensor";
        let result = get_user_from_sensor(sensor_api_id);
        assert!(
            result.is_err(),
            "Expected error when getting user from nonexistent sensor"
        );
        if let Err(crate::db::Error::DataBaseError(diesel::result::Error::NotFound)) = result {
            // Expected error for nonexistent sensor
        } else {
            assert!(false, "Expected a NotFound error, but got: {:?}", result);
        }
    }

    #[test]
    fn test_test_db_pool() {
        let result = test_db_pool();
        assert!(
            result.is_ok(),
            "Failed to establish test database connection"
        );
    }

    #[test]
    fn test_insert_aht10data_then_delete() {
        let now = chrono::Utc::now().naive_utc();

        let new_data = models::NewAht10Data {
            sensor: "94a990533d761111111111111111111111111111".into(),
            serialized_data: "{\"temperature\": 22.5, \"humidity\": 45.0}",
            added_at: now, // Example timestamp
        };

        println!("INSERTING DATA");
        save_new_aht10_data(new_data).expect("Data should be inserted correctly");
        println!("DATA INSERTED");

        let mut db_conn = get_db_pool().expect("Failed to get database connection");

        use crate::schema::aht10data::dsl::*;
        let count = diesel::delete(aht10data.filter(added_at.eq(now)))
            .execute(&mut db_conn)
            .expect("Failed to delete AHT10 data");

        assert_eq!(
            count, 1,
            "Expected to delete 1 record, but deleted {}",
            count
        );
    }

    #[test]
    fn test_insert_aht10data_unexistent_sensor() {
        let new_data = models::NewAht10Data {
            sensor: "94a990533d761111111111111111111111111111e".into(),
            serialized_data: "{\"temperature\": 22.5, \"humidity\": 45.0}",
            added_at: chrono::Utc::now().naive_utc(),
        };

        let result = save_new_aht10_data(new_data);
        assert!(
            result.is_err(),
            "Expected error when inserting data for a nonexistent user"
        );
        if let Err(crate::db::Error::DataBaseError(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
            _,
        ))) = result
        {
            // Expected error for inserting data for a nonexistent user
        } else {
            panic!("Expected a NotFound error, but got: {:?}", result);
        }
    }
    #[test]
    fn test_query_aht10_data() {
        let query = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string("94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .expect("Valid API ID"),
            sensor_api_id: ApiId::from_string("94a990533d761111111111111111111111111111")
                .expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };

        let result = query_aht10_data(query).expect("Failed to query AHT10 data for existing user");
        assert_eq!(
            result.1, 0,
            "Expected no deserialization errors, found {}",
            result.1
        );
        assert!(
            result.0.len() == 10,
            "Expected 10 records, found {}",
            result.0.len()
        );
    }

    #[test]
    fn test_query_aht10_data_nonexistent_user() {
        let query = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string("abcdef9999abcdef9999abcdef9999abcdef9999")
                .expect("Valid API ID"),
            sensor_api_id: ApiId::from_string("94a990533d762222222222222222222222222222")
                .expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };
        let result =
            query_aht10_data(query).expect("Failed to query AHT10 data for nonexistent user");
        assert!(
            result.0.is_empty(),
            "Expected no records for nonexistent user, found {}",
            result.0.len()
        );
        assert_eq!(
            result.1, 0,
            "Expected no deserialization errors, found {}",
            result.1
        );
    }

    #[test]
    fn test_insert_scd4xdata_then_delete() {
        let now = chrono::Utc::now().naive_utc();

        let new_data = models::NewScd4xData {
            sensor: "94a990533d762222222222222222222222222222".into(),
            serialized_data: "{\"co2\": 420, \"temperature\": 21.5, \"humidity\": 40.2}",
            added_at: now, // Example timestamp
        };

        save_new_scd4x_data(new_data).expect("Data should be inserted correctly");

        let mut db_conn = get_db_pool().expect("Failed to get database connection");

        use crate::schema::scd4xdata::dsl::*;
        let count = diesel::delete(scd4xdata.filter(added_at.eq(now)))
            .execute(&mut db_conn)
            .expect("Failed to delete SCD4X data");

        assert_eq!(
            count, 1,
            "Expected to delete 1 record, but deleted {}",
            count
        );
    }

    #[test]
    fn test_insert_scd4xdata_unexistent_sensor() {
        let new_data = models::NewScd4xData {
            sensor: "94a990533d762222222222222222222222222222e".into(),
            serialized_data: "{\"co2\": 420, \"temperature\": 21.5, \"humidity\": 40.2}",
            added_at: chrono::Utc::now().naive_utc(),
        };

        let result = save_new_scd4x_data(new_data);
        assert!(
            result.is_err(),
            "Expected error when inserting data for a nonexistent user"
        );
        if let Err(crate::db::Error::DataBaseError(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::ForeignKeyViolation,
            _,
        ))) = result
        {
            // Expected error for inserting data for a nonexistent user
        } else {
            panic!(
                "Expected a ForeignKeyViolation error, but got: {:?}",
                result
            );
        }
    }

    #[test]
    fn test_query_scd4x_data() {
        let query = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string("94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .expect("Valid API ID"),
            sensor_api_id: ApiId::from_string("94a990533d762222222222222222222222222222")
                .expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };

        let result = query_scd4x_data(query).expect("Failed to query SCD4X data for existing user");
        assert_eq!(
            result.1, 0,
            "Expected no deserialization errors, found {}",
            result.1
        );
        assert!(
            result.0.len() == 10,
            "Expected 10 records, found {}",
            result.0.len()
        );
    }

    #[test]
    fn test_query_scd4x_data_nonexistent_user() {
        let query = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string("abcdef9999abcdef9999abcdef9999abcdef9999")
                .expect("Valid API ID"),
            sensor_api_id: ApiId::from_string("94a990533d762222222222222222222222222222")
                .expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };
        let result =
            query_aht10_data(query).expect("Failed to query SCD4X data for nonexistent user");
        assert!(
            result.0.is_empty(),
            "Expected no records for nonexistent user, found {}",
            result.0.len()
        );
        assert_eq!(
            result.1, 0,
            "Expected no deserialization errors, found {}",
            result.1
        );
    }
}
