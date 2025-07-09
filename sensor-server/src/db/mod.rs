use diesel::connection::SimpleConnection;
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;

use once_cell::sync::Lazy;
use sensor_lib::api;
use sensor_lib::api::endpoints::get_sensor_data::GetSensorRequestBody;
use sensor_lib::api::model::sensor_kind::SensorKind;

use crate::models;

use diesel::r2d2::Error as DieselPoolError;

type FailedDeserialize = usize;

#[derive(Debug)]
pub enum Error {
    DataBaseError(diesel::result::Error),
    DataBaseConnectionError,
    SerializationError(serde_json::Error),
    DeviceIdInvalid,
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

type DbPool = r2d2::Pool<ConnectionManager<diesel::SqliteConnection>>;

static DB_POOL: Lazy<DbPool> = Lazy::new(|| {
    dotenv().expect("Failed to read .env file");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<diesel::SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
});

fn get_db_pool()
-> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, crate::db::Error> {
    let mut db: r2d2::PooledConnection<ConnectionManager<SqliteConnection>> = DB_POOL
        .get()
        .map_err(|_| crate::db::Error::DataBaseConnectionError)?;

    db.batch_execute(
        "PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;",
    )?;

    Ok(db)
}

pub fn test_db_pool() -> Result<(), ()> {
    DB_POOL.get().map_err(|_| ())?;
    log::info!("Test database connection established successfully");
    Ok(())
}

pub fn query_aht10_data(
    query: GetSensorRequestBody,
) -> Result<(Vec<api::model::aht10_data::Aht10Data>, FailedDeserialize), Error> {
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
    let sensor_ = &query.sensor_api_id;
    let api_id = &query.user_api_id;

    let mut failed_deserialize: usize = 0;

    let vec = aht10data_table
        .filter(aht10data::sensor.eq(sensor_))
        .inner_join(user_sensors_table)
        .inner_join(user_places_table.on(user_places::id.eq(user_sensors::place)))
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(api_id))
        .filter(aht10data::added_at.le(max_date.unwrap_or(i32::MAX)))
        .filter(aht10data::added_at.ge(min_date.unwrap_or(0)))
        .select(models::Aht10Data::as_select())
        .load::<models::Aht10Data>(&mut db_conn)?
        .into_iter()
        .map(|data| {
            let deserialized = serde_json::from_str::<api::model::aht10_data::Aht10Data>(&data.serialized_data);

            match deserialized {
                Ok(value) => Some(value),
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
    query: GetSensorRequestBody,
) -> Result<(Vec<api::model::scd4x_data::Scd4xData>, FailedDeserialize), Error> {
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
    let sensor_ = &query.sensor_api_id;
    let api_id = &query.user_api_id;

    let mut failed_deserialize: usize = 0;

    let vec = scd4xdata_table
        .filter(scd4xdata::sensor.eq(sensor_))
        .inner_join(user_sensors_table)
        .inner_join(user_places_table.on(user_places::id.eq(user_sensors::place)))
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(api_id))
        .filter(scd4xdata::added_at.le(max_date.unwrap_or(i32::MAX)))
        .filter(scd4xdata::added_at.ge(min_date.unwrap_or(0)))
        .select(models::Scd4xData::as_select())
        .load::<models::Scd4xData>(&mut db_conn)?
        .into_iter()
        .map(|data| {
            let deserialized =
                serde_json::from_str::<api::model::scd4x_data::Scd4xData>(&data.serialized_data);

            match deserialized {
                Ok(value) => Some(value),
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

    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };

    let mut db_conn = get_db_pool()?;

    let user_id_result = user_sensors_table
        .inner_join(user_places_table)
        .filter(user_sensors::api_id.eq(sensor_api_id))
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
    user_place_id_: i32,
) -> Result<bool, crate::db::Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let count = user_places_table
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(user_api_id))
        .filter(user_places::id.eq(user_place_id_))
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

    let mut db_conn: r2d2::PooledConnection<ConnectionManager<SqliteConnection>> = get_db_pool()?;

    diesel::insert_into(aht10data::table)
        .values(data)
        .execute(&mut db_conn)?;
    Ok(())
}

pub fn save_new_scd4x_data(data: models::NewScd4xData<'_>) -> Result<(), Error> {
    use crate::schema::scd4xdata;

    log::info!("Saving new SCD4X data: {:?}", data);

    let mut db_conn: r2d2::PooledConnection<ConnectionManager<SqliteConnection>> = get_db_pool()?;

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
    let device_id = device_id.to_uppercase();

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

pub fn new_sensor(
    user_api_id: &str,
    sensor_kind: SensorKind,
    user_place_id_: i32,
    device_id: &str,
) -> Result<String, Error> {
    use crate::schema::user_sensors;

    if !user_api_id_matches_place_id(user_api_id, user_place_id_)? {
        return Err(Error::DataBaseError(diesel::result::Error::NotFound));
    }

    let mut db_conn = get_db_pool()?;

    let user_sensor_api_id =
        generate_sensor_api_id(user_api_id, &sensor_kind, user_place_id_, device_id);
    let user_sensor_api_id = hex::encode(user_sensor_api_id?);

    let new_sensor = models::NewUserSensor {
        api_id: &user_sensor_api_id,
        place: user_place_id_,
        kind: sensor_kind as i32,
        last_measurement: 0,
        device_id,
    };

    diesel::insert_into(user_sensors::table)
        .values(new_sensor)
        .execute(&mut db_conn)?;

    Ok(user_sensor_api_id)
}

pub fn get_login(username: &str, hashed_password: &str) -> Result<String, Error> {
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};
    let mut db_conn = get_db_pool()?;
    let user = users_table
        .filter(users::username.eq(username))
        .filter(users::hashed_password.eq(hashed_password))
        .select(users::api_id)
        .first::<String>(&mut db_conn)
        .map_err(|_| Error::DataBaseError(diesel::result::Error::NotFound))?;
    Ok(user)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_login() {
        let username = "testuser";
        let hashed_password = "hashedpassword123"; // Replace with actual hashed password

        let result = get_login(username, hashed_password);
        assert!(result.is_ok(), "Failed to get login: {:?}", result.err());
        let user_uuid = result.unwrap();
        assert!(!user_uuid.is_empty(), "User UUID should not be empty");
    }

    #[test]
    fn test_get_login_nonexistent() {
        let username = "nonexistentuser";
        let hashed_password = "hashedpassword123"; // Replace with actual hashed password

        let result = get_login(username, hashed_password);
        assert!(
            result.is_err(),
            "Expected error when getting login for nonexistent user"
        );
        if let Err(crate::db::Error::DataBaseError(diesel::result::Error::NotFound)) = result {
            // Expected error for nonexistent user
        } else {
            panic!("Expected a NotFound error, but got: {:?}", result);
        }
    }

    #[test]
    fn test_get_login_incorrect_password() {
        let username = "testuser";
        let hashed_password = "wronghashedpassword"; // Replace with actual hashed password

        let result = get_login(username, hashed_password);
        assert!(
            result.is_err(),
            "Expected error when getting login with incorrect password"
        );
        if let Err(crate::db::Error::DataBaseError(diesel::result::Error::NotFound)) = result {
            // Expected error for incorrect password
        } else {
            panic!("Expected a NotFound error, but got: {:?}", result);
        }
    }

    #[test]
    fn test_generate_sensor_id() {
        let user_uuid = "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let sensor_kind = SensorKind::Aht10;
        let user_place_id = 1; // Assuming this is a valid place ID for the test user
        let device_ids_ok = [
            "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAB",
            "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAC",
            "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAD",
        ];
        let device_ids_invalid = [
            "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAABhola", // Too long invalid
            "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAhola",     // Invalid chars
            "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAA",         // Too short
            "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAABAAAA", // Too long valid
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
        let user_uuid = "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let sensor_kind = SensorKind::Aht10;
        let user_place_id = 1; // Assuming this is a valid place ID for the test user
        let device_id = "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAB";

        let result = new_sensor(user_uuid, sensor_kind, user_place_id, device_id);
        assert!(result.is_ok(), "Failed to create new sensor");

        let sensor_api_id = result.unwrap();
        assert!(
            !sensor_api_id.is_empty(),
            "Sensor API ID should not be empty"
        );

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

        diesel::delete(user_sensors_table.filter(user_sensors::api_id.eq(&sensor_api_id)))
            .execute(&mut db_conn)
            .expect("Failed to delete sensor");
    }

    #[test]
    fn test_user_api_id_matches_sensor_api_id() {
        let user_api_id = "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let sensor_api_id = "94a990533d761111111111111111111111111111";
        let result = user_api_id_matches_sensor_api_id(user_api_id, sensor_api_id)
            .expect("Failed to check if user UUID matches sensor API ID");
        assert!(
            result,
            "Expected user UUID to match sensor API ID, but it did not"
        );
    }

    #[test]
    fn test_user_uuid_matches_sensor_api_id_nonexistent() {
        let user_uuid = "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA";
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
        let user_uuid = "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA";
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
        let user_uuid = "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA";
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
    fn test_embeed_run_migrations() {
        use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
        let mut db_conn = get_db_pool().expect("Failed to get database connection");
        db_conn
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        log::info!("Migrations ran successfully");
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
        let now = chrono::Utc::now().timestamp() as i32;

        let new_data = models::NewAht10Data {
            sensor: "94a990533d761111111111111111111111111111".into(),
            serialized_data: "{\"temperature\": 22.5, \"humidity\": 45.0}",
            added_at: now, // Example timestamp
        };

        let result = save_new_aht10_data(new_data);
        assert!(result.is_ok(), "Failed to insert AHT10 data");

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
            added_at: 1633036800, // Example timestamp
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
        let query = GetSensorRequestBody {
            user_api_id: "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
            sensor_api_id: "94a990533d761111111111111111111111111111".into(),
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
        let query = GetSensorRequestBody {
            user_api_id: "nonexistent-user_api_id".into(),
            sensor_api_id: "94a990533d761111111111111111111111111111".into(),
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
        let now = chrono::Utc::now().timestamp() as i32;

        let new_data = models::NewScd4xData {
            sensor: "94a990533d762222222222222222222222222222".into(),
            serialized_data: "{\"co2\": 420, \"temperature\": 21.5, \"humidity\": 40.2}",
            added_at: now, // Example timestamp
        };

        let result = save_new_scd4x_data(new_data);
        assert!(result.is_ok(), "Failed to insert SCD4X data");

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
            added_at: 1633036800, // Example timestamp
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
        let query = GetSensorRequestBody {
            user_api_id: "94a990533d76AAAAAAAAAAAAAAAAAAAAAAAAAAAA".into(),
            sensor_api_id: "94a990533d762222222222222222222222222222".into(),
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
        let query = GetSensorRequestBody {
            user_api_id: "nonexistent-user_api_id".into(),
            sensor_api_id: "94a990533d762222222222222222222222222222".into(),
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
