use chrono::{NaiveDateTime, TimeZone};
use diesel::dsl::count_star;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;

use once_cell::sync::Lazy;
use sensor_lib::api;
use sensor_lib::api::endpoints::get_sensor_data::GetSensorDataRequestBody;
use sensor_lib::api::endpoints::register::RegisterRequestBody;
use sensor_lib::api::model::api_id::{self, ApiId};
use sensor_lib::api::model::color_palette::{PlaceColor, SensorColor};
use sensor_lib::api::model::sensor_kind::SensorKind;

use crate::models::{self, SensorData, UserPlace, UserSensor};

use r2d2::Error as DieselPoolError;

type FailedDeserialize = usize;

#[derive(Debug)]
pub enum Error {
    DataBaseError(diesel::result::Error),
    DataBaseConnectionError,
    SerializationError(serde_json::Error),
    DeviceIdInvalid,
    Inconsistency(String),
    ApiIdError(api_id::Error),
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
) -> Result<(SensorKind, Vec<SensorData>), Error> {
    use crate::schema::{
        sensor_data::dsl as sensor_data, sensor_data::dsl::sensor_data as sensor_data_table,
    };
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
        .inner_join(user_sensors_table.on(user_places::api_id.eq(user_sensors::place)))
        .filter(user_sensors::api_id.eq(&query.sensor_api_id.as_str()))
        .select(models::UserSensor::as_select())
        .load::<models::UserSensor>(&mut db_conn)?;

    let kind = match r.first() {
        Some(s) => match SensorKind::from_i32(s.kind) {
            Some(k) => k,
            None => Err(Error::Inconsistency(String::from(
                "SensorKind in DB is wrong",
            )))?,
        },
        None => Err(Error::NotFound)?,
    };

    let max_date = query
        .added_at_upper
        .and_then(|secs| (secs as i64).checked_mul(1_000))
        .and_then(|millis| chrono::Utc::timestamp_millis_opt(&chrono::Utc, millis).earliest())
        .and_then(|dt| Some(dt.naive_utc()));
    let min_date = query
        .added_at_lower
        .and_then(|secs| (secs as i64).checked_mul(1_000))
        .and_then(|millis| chrono::Utc::timestamp_millis_opt(&chrono::Utc, millis).earliest())
        .and_then(|dt| Some(dt.naive_utc()));

    let vec = sensor_data_table
        .filter(sensor_data::sensor.eq(query.sensor_api_id.as_str()))
        .inner_join(user_sensors_table)
        .inner_join(user_places_table.on(user_places::api_id.eq(user_sensors::place)))
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(query.user_api_id.as_str()))
        .filter(sensor_data::added_at.le(max_date.unwrap_or(NaiveDateTime::MAX)))
        .filter(
            sensor_data::added_at.ge(min_date.unwrap_or(
                NaiveDateTime::from_timestamp_opt(0, 0)
                    .expect("(secs: 0, nsecs: 0) should be valid timestamp"),
            )),
        )
        .select(models::SensorData::as_select())
        .load::<models::SensorData>(&mut db_conn)?;

    Ok((kind, vec))
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
    user_api_id: &ApiId,
    sensor_api_id: &ApiId,
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
        .inner_join(user_places_table.on(user_places::api_id.eq(user_sensors::place)))
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(user_api_id.to_string()))
        .filter(user_sensors::api_id.eq(sensor_api_id.to_string()))
        .select(count_star())
        .first::<i64>(&mut db_conn)?;

    Ok(count > 0)
}

pub fn user_api_id_matches_place_id(
    user_api_id: &ApiId,
    user_place_id: &ApiId,
) -> Result<bool, crate::db::Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    let count = user_places_table
        .inner_join(users_table.on(users::username.eq(user_places::user)))
        .filter(users::api_id.eq(user_api_id.as_str()))
        .filter(user_places::api_id.eq(user_place_id.as_str()))
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

pub fn save_new_sensor_data(data: models::NewSensorData<'_>) -> Result<(), Error> {
    use crate::schema::sensor_data;

    log::info!("Saving new AHT10 data: {:?}", data);

    let mut db_conn: r2d2::PooledConnection<ConnectionManager<PgConnection>> = get_db_pool()?;

    diesel::insert_into(sensor_data::table)
        .values(data)
        .execute(&mut db_conn)?;
    Ok(())
}

pub fn generate_sensor_api_id(
    user_api_id: &ApiId,
    sensor_kind: &SensorKind,
    user_place_api_id: &ApiId,
    device_id: &ApiId,
) -> Result<[u8; 20], Error> {
    let device_id = device_id;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(user_api_id.as_str().as_bytes());
    hasher.update(sensor_kind.as_str().as_bytes());
    hasher.update(user_place_api_id.as_str().as_bytes());
    hasher.update(device_id.as_str().as_bytes());
    let hash = hasher.finalize();
    let mut sensor_id = [0u8; 20];
    sensor_id.copy_from_slice(&hash[..20]);
    Ok(sensor_id)
}

// Returns sensor api id if already present on DB
pub fn sensor_exists(
    user_api_id: &ApiId,
    user_place_id: &ApiId,
    device_id: &ApiId,
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
        .filter(users::api_id.eq(user_api_id.as_str()))
        .inner_join(user_places_table)
        .filter(user_places::api_id.eq(user_place_id.as_str()))
        .inner_join(user_sensors_table.on(user_sensors::device_id.eq(device_id.as_str())))
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
    user_api_id: &ApiId,
    sensor_kind: SensorKind,
    user_place_api_id: &ApiId,
    device_id: &ApiId,
    name: &str,
    description: Option<&str>,
    color: SensorColor,
) -> Result<String, Error> {
    use crate::schema::user_sensors;

    let user_api_id = user_api_id;

    if !user_api_id_matches_place_id(&user_api_id, &user_place_api_id)? {
        return Err(Error::NotFound);
    }

    let mut db_conn = get_db_pool()?;

    let user_sensor_api_id =
        generate_sensor_api_id(&user_api_id, &sensor_kind, &user_place_api_id, device_id);
    let user_sensor_api_id = hex::encode(user_sensor_api_id?);

    let new_sensor = models::NewUserSensor {
        api_id: &user_sensor_api_id,
        place: user_place_api_id.as_str().to_string(),
        kind: sensor_kind as i32,
        last_measurement: NaiveDateTime::from_timestamp(0, 0),
        device_id: device_id.as_str(),
        name,
        description,
        color: color.as_str(),
    };

    diesel::insert_into(user_sensors::table)
        .values(new_sensor)
        .execute(&mut db_conn)?;

    Ok(user_sensor_api_id)
}

pub fn new_place(
    user_api_id: &ApiId,
    username: &str,
    place_name: &str,
    place_description: Option<&str>,
    color: PlaceColor,
) -> Result<ApiId, Error> {
    use crate::schema::user_places;

    let user_api_id = user_api_id;

    // Check for user existence
    get_user(username, user_api_id.as_str())?;

    let mut db_conn = get_db_pool()?;

    let now = chrono::Utc::now().naive_utc();
    let api_id = ApiId::random();

    let new_place = models::NewUserPlace {
        api_id: api_id.as_str(),
        user: username,
        name: place_name,
        description: place_description,
        created_at: now,
        updated_at: now,
        color: color.as_str(),
    };

    diesel::insert_into(user_places::table)
        .values(new_place)
        .execute(&mut db_conn)?;

    Ok(api_id)
}

pub fn get_login(username: &str, hashed_password: &str) -> Result<Option<ApiId>, Error> {
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};
    let mut db_conn = get_db_pool()?;
    let user_api_id = users_table
        .filter(users::username.eq(username))
        .filter(users::hashed_password.eq(hashed_password))
        .select(users::api_id)
        .first::<String>(&mut db_conn);

    match user_api_id {
        Ok(user_api_id) => match ApiId::from_string(&user_api_id) {
            Ok(id) => Ok(Some(id)),
            Err(e) => Err(Error::ApiIdError(e)),
        },
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
    let api_id = ApiId::random();
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

pub fn delete_sensor(user_api_id: &ApiId, sensor_api_id: &ApiId) -> Result<(), Error> {
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };

    // check for it
    if !user_api_id_matches_sensor_api_id(user_api_id, sensor_api_id)? {
        return Err(Error::NotFound);
    }

    match diesel::delete(user_sensors_table.filter(user_sensors::api_id.eq(sensor_api_id.as_str())))
        .execute(&mut get_db_pool()?)
    {
        Ok(s) => {
            if s > 1 {
                log::error!("Many sensors affected by delete on single api_id");
                Ok(())
            } else if s == 0 {
                log::warn!("No user found to delete after we checked for it");
                Err(Error::NotFound)
            } else {
                Ok(()) // User was deleted successfully
            }
        }
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                log::warn!("No user_sensor found to delete after we checked for it");
                Err(Error::NotFound)
            }
            e => Err(Error::DataBaseError(e)),
        },
    }
}

pub fn delete_user(api_id: &ApiId) -> Result<(), Error> {
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let mut db_conn = get_db_pool()?;

    match diesel::delete(users_table.filter(users::api_id.eq(api_id.as_str())))
        .execute(&mut db_conn)
    {
        Ok(s) => {
            if s > 1 {
                log::error!("Many users affected by delete on single api_id");
                Ok(())
            } else if s == 0 {
                log::warn!("No user found to delete after we checked for it");
                Err(Error::NotFound)
            } else {
                Ok(()) // User was deleted successfully
            }
        }
        Err(e) => match e {
            diesel::result::Error::NotFound => Err(Error::NotFound),
            _ => Err(Error::DataBaseError(e)),
        },
    }
}

pub fn delete_place(user_api_id: &ApiId, place_api_id: &ApiId) -> Result<(), Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };

    // check for it
    if !user_api_id_matches_place_id(user_api_id, place_api_id)? {
        return Err(Error::NotFound);
    }

    match diesel::delete(user_places_table.filter(user_places::api_id.eq(place_api_id.as_str())))
        .execute(&mut get_db_pool()?)
    {
        Ok(s) => {
            if s > 1 {
                log::error!("Many places affected by delete on single api_id");
                Ok(())
            } else if s == 0 {
                log::warn!("No place found to delete after we checked for it");
                Err(Error::NotFound)
            } else {
                Ok(()) // Place was deleted successfully
            }
        }
        Err(e) => match e {
            diesel::result::Error::NotFound => {
                log::warn!("No user_place found to delete after we checked for it");
                Err(Error::NotFound)
            }
            e => Err(Error::DataBaseError(e)),
        },
    }
}

pub fn get_user_sensors(
    username: &str,
    user_api_id: &str,
) -> Result<Vec<(UserSensor, UserPlace)>, Error> {
    use crate::schema::{
        user_places::dsl as user_places, user_places::dsl::user_places as user_places_table,
    };
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    let r = users_table
        .filter(
            users::api_id
                .eq(user_api_id)
                .and(users::username.eq(username)),
        )
        .inner_join(user_places_table)
        .inner_join(user_sensors_table.on(user_sensors::place.eq(user_places::api_id)))
        .select((UserSensor::as_select(), UserPlace::as_select()))
        .load::<(UserSensor, UserPlace)>(&mut get_db_pool()?);

    match r {
        Ok(sensors) => Ok(sensors),
        Err(e) => match e {
            diesel::result::Error::NotFound => Err(Error::NotFound),
            e => Err(Error::DataBaseError(e)),
        },
    }
}

pub fn get_user(username: &str, user_api_id: &str) -> Result<models::User, Error> {
    use crate::schema::{users::dsl as users, users::dsl::users as users_table};

    match users_table
        .filter(
            users::api_id
                .eq(user_api_id)
                .and(users::username.eq(username)),
        )
        .select(models::User::as_select())
        .first::<models::User>(&mut get_db_pool()?)
    {
        Ok(s) => Ok(s),
        Err(e) => match e {
            diesel::result::Error::NotFound => Err(Error::NotFound),
            _ => Err(Error::DataBaseError(e)),
        },
    }
}

pub fn get_user_email(username: &str, user_api_id: &str) -> Result<String, Error> {
    get_user(username, user_api_id).map(|u| u.email)
}

pub fn update_sensor_last_measurement(
    updated_at: NaiveDateTime,
    sensor_api_id: &str,
) -> Result<(), Error> {
    use crate::schema::{
        user_sensors::dsl as user_sensors, user_sensors::dsl::user_sensors as user_sensors_table,
    };
    match diesel::update(user_sensors_table.find(sensor_api_id))
        .set(user_sensors::last_measurement.eq(updated_at))
        .execute(&mut get_db_pool()?)
    {
        Ok(n) => match n {
            0 => Err(Error::NotFound),
            1 => Ok(()),
            _ => Err(Error::Inconsistency(
                "Many user_sensors affected by update on single api_id".to_string(),
            )),
        },
        Err(e) => Err(Error::DataBaseError(e)),
    }
}

#[cfg(test)]
mod tests {

    use chrono::SubsecRound;

    use super::*;

    const VALID_USER_API_ID: &'static str = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const VALID_AHT10_API_ID: &'static str = "94a990533d761111111111111111111111111111";
    const VALID_SCD41_API_ID: &'static str = "94a990533d762222222222222222222222222222";
    const VALID_PLACE_ID_1: &'static str = "94a990533d76ffaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const VALID_PLACE_ID_2: &'static str = "94a990533d76fffaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn test_sensor_exists() {
        let user_api_id = ApiId::from_string(VALID_USER_API_ID).unwrap();
        let user_place_id = ApiId::from_string(VALID_PLACE_ID_2).unwrap();
        let device_id = ApiId::from_string("94a990533d770000000000000000000000000000").unwrap();

        let result = sensor_exists(&user_api_id, &user_place_id, &device_id);
        assert!(
            result.is_ok(),
            "Failed to check if sensor exists: {:?}",
            result.err()
        );
        let exists = result.unwrap();
        assert!(exists.is_some(), "Expected sensor to exist");
    }

    #[test]
    fn test_sensor_exists_unexistent() {
        let user_api_id = ApiId::from_string(VALID_USER_API_ID).unwrap();
        let user_place_id = ApiId::from_string(VALID_PLACE_ID_2).unwrap();
        let device_id = ApiId::random();

        let result = sensor_exists(&user_api_id, &user_place_id, &device_id);
        assert!(
            result.is_ok(),
            "Failed to check if sensor exists: {:?}",
            result.err()
        );
        let exists = result.unwrap();
        assert!(exists.is_none(), "Expected sensor to not exist");
    }

    #[test]
    fn test_sensor_exists_unexistent_place() {
        let user_api_id = ApiId::from_string(VALID_USER_API_ID).unwrap();
        let user_place_id = ApiId::random();
        let device_id = ApiId::from_string(VALID_SCD41_API_ID).unwrap();

        let result = sensor_exists(&user_api_id, &user_place_id, &device_id);
        assert!(
            result.is_ok(),
            "Failed to check if sensor exists: {:?}",
            result.err()
        );
        let exists = result.unwrap();
        assert!(exists.is_none(), "Expected sensor to not exist");
    }

    #[test]
    fn test_sensor_exists_unexistent_user() {
        let user_api_id = ApiId::random();
        let user_place_id = ApiId::from_string(VALID_USER_API_ID).unwrap();
        let device_id = ApiId::from_string(VALID_SCD41_API_ID).unwrap();

        let result = sensor_exists(&user_api_id, &user_place_id, &device_id);
        assert!(
            result.is_ok(),
            "Failed to check if sensor exists: {:?}",
            result.err()
        );
        let exists = result.unwrap();
        assert!(exists.is_none(), "Expected sensor to not exist");
    }

    #[test]
    fn test_get_user_email() {
        let username = "testuser";
        let user_api_id = VALID_USER_API_ID;

        let result = get_user_email(username, user_api_id);
        assert!(
            result.is_ok(),
            "Failed to get user email: {:?}",
            result.err()
        );
        let email = result.unwrap();
        println!("User email: {}", email);
        assert!(!email.is_empty(), "Expected non-empty email");
    }

    #[test]
    fn test_get_user_sensors() {
        let username = "testuser";
        let user_api_id = VALID_USER_API_ID;

        let result = get_user_sensors(username, user_api_id);
        assert!(
            result.is_ok(),
            "Failed to get user sensors: {:?}",
            result.err()
        );
        let sensors = result.unwrap();
        println!("User sensors: {:?}", sensors);
        assert!(!sensors.is_empty(), "Expected non-empty sensor list");
    }

    #[test]
    fn test_new_user_then_delete() {
        let query = RegisterRequestBody {
            username: "testuser_not_exists".to_string(),
            hashed_password: "ae5deb822e0d71992900471a7199d0d95b8e7c9d05c40a8245a281fd2c1d6684"
                .to_string(),
            email: "testuser_not_exists@example.com".to_string(),
        };

        let r = new_user(query);
        let user_api_id = r.expect("Should be able to create new user with DB");

        // Delete
        match delete_user(&user_api_id) {
            Ok(()) => assert!(true, "User should be deleted successfully"),
            Err(_) => panic!("Should be able to delete user with DB"),
        }
    }

    #[test]
    fn test_get_login() {
        let username = "testuser";
        let hashed_password = "ae5deb822e0d71992900471a7199d0d95b8e7c9d05c40a8245a281fd2c1d6684";

        let result = get_login(username, hashed_password);
        assert!(result.is_ok(), "Failed to get login: {:?}", result.err());
        let _ = result.unwrap().expect("Should be Some");
    }

    #[test]
    fn test_get_login_nonexistent() {
        let username = "nonexistentuser";
        let hashed_password = "ae5deb822e0d71992900471a7199d0d95b8e7c9d05c40a8245a281fd2c1d6684";

        let result = get_login(username, hashed_password);
        assert!(
            result.is_ok_and(|none| none.is_none()),
            "Expected None when getting login with incorrect password"
        );
    }

    #[test]
    fn test_get_login_incorrect_password() {
        let username = "testuser";
        let hashed_password = "wronghashedpassword";

        let result = get_login(username, hashed_password);
        assert!(
            result.is_ok_and(|none| none.is_none()),
            "Expected None when getting login with incorrect password"
        );
    }

    #[test]
    fn test_generate_sensor_id() {
        let user_api_id = VALID_USER_API_ID;
        let sensor_kind = SensorKind::Aht10;
        let user_place_id = VALID_PLACE_ID_1; // Assuming this is a valid place ID for the test user
        let device_ids_ok = [
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaab",
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaac",
            "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaad",
        ];

        for device_id in device_ids_ok {
            let sensor_api_id = generate_sensor_api_id(
                &ApiId::from_string(user_api_id).unwrap(),
                &sensor_kind,
                &ApiId::from_string(user_place_id).unwrap(),
                &ApiId::from_string(device_id).unwrap(),
            )
            .expect("Failed to generate sensor ID");
            assert_eq!(
                sensor_api_id.len(),
                20,
                "Sensor ID should be 20 bytes long, got {} for {:?}",
                sensor_api_id.len(),
                sensor_api_id
            );
        }
    }

    #[test]
    fn test_new_sensor_then_delete() {
        let user_api_id = &ApiId::from_string(VALID_USER_API_ID).unwrap();
        let sensor_kind = SensorKind::Aht10;
        let user_place_id = &ApiId::from_string(VALID_PLACE_ID_1).unwrap(); // Assuming this is a valid place ID for the test user
        let device_id = &ApiId::from_string("94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaab").unwrap();

        let result = new_sensor(
            user_api_id,
            sensor_kind,
            user_place_id,
            device_id,
            "Sensor 1",
            None,
            SensorColor::HEX_2132DB,
        )
        .expect("Should be able to create new sensor with DB");
        assert!(!result.is_empty(), "Sensor API ID should not be empty");

        let errored_result = new_sensor(
            user_api_id,
            sensor_kind,
            user_place_id,
            device_id,
            "Sensor 1",
            None,
            SensorColor::HEX_2132DB,
        );
        assert!(
            errored_result.is_err(),
            "Expected error when creating sensor with existing DEVICE ID"
        );

        let errored_result = new_sensor(
            &ApiId::random(),
            sensor_kind,
            user_place_id,
            &ApiId::random(),
            "Sensor 2",
            None,
            SensorColor::HEX_2132DB,
        );
        assert!(
            errored_result.is_err(),
            "Expected error when creating sensor for nonexistent user"
        );

        let errored_result = new_sensor(
            user_api_id,
            sensor_kind,
            &ApiId::random(), // Assuming this is an invalid place ID
            &ApiId::random(),
            "Sensor 3",
            None,
            SensorColor::HEX_2132DB,
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
        let user_api_id = VALID_USER_API_ID;
        let sensor_api_id = "abc36768cf4d927e267a72ac1cb8108693bdafd1";
        let matches = user_api_id_matches_sensor_api_id(
            &ApiId::from_string(user_api_id).unwrap(),
            &ApiId::from_string(sensor_api_id).unwrap(),
        )
        .expect("Failed to check if user UUID matches sensor API ID");
        assert!(
            matches,
            "Expected user UUID to match sensor API ID, but it did not"
        );
    }

    #[test]
    fn test_user_uuid_matches_place_id() {
        let uaser_api_id = &ApiId::from_string(VALID_USER_API_ID).unwrap();
        let user_place_id = &ApiId::from_string(VALID_PLACE_ID_1).unwrap();
        let result = user_api_id_matches_place_id(uaser_api_id, user_place_id);
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
        let user_api_id = &ApiId::from_string(VALID_USER_API_ID).unwrap();
        let user_place_id = &ApiId::random(); // Assuming this is an invalid place ID
        let result = user_api_id_matches_place_id(user_api_id, user_place_id).expect("no error");
        assert!(
            !result,
            "Expected user UUID to not match nonexistent place ID, but it did"
        );
    }

    #[test]
    fn test_get_user_from_sensor() {
        let sensor_api_id = VALID_AHT10_API_ID;
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

        let new_data = models::NewSensorData {
            sensor: VALID_AHT10_API_ID.into(),
            serialized_data: "{\"temperature\": 22.5, \"humidity\": 45.0}",
            added_at: now, // Example timestamp
        };

        println!("INSERTING DATA");
        save_new_sensor_data(new_data).expect("Data should be inserted correctly");
        println!("DATA INSERTED");

        let mut db_conn = get_db_pool().expect("Failed to get database connection");

        use crate::schema::sensor_data::dsl::*;
        let count = diesel::delete(sensor_data.filter(added_at.eq(now)))
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
        let new_data = models::NewSensorData {
            sensor: "94a990533d761111111111111111111111111111e".into(),
            serialized_data: "{\"temperature\": 22.5, \"humidity\": 45.0}",
            added_at: chrono::Utc::now().naive_utc(),
        };

        let result = save_new_sensor_data(new_data);
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
            user_api_id: ApiId::from_string(VALID_USER_API_ID).expect("Valid API ID"),
            sensor_api_id: ApiId::from_string(VALID_AHT10_API_ID).expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };

        let result =
            query_sensor_data(query).expect("Failed to query AHT10 data for existing user");

        assert_eq!(result.0, SensorKind::Aht10, "Sensor kind must be the same");
        assert!(
            result.1.len() == 10,
            "Expected 10 records, found {}",
            result.1.len()
        );
    }

    #[test]
    fn test_query_aht10_data_nonexistent_user() {
        let query = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string("abcdef9999abcdef9999abcdef9999abcdef9999")
                .expect("Valid API ID"),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };
        let result = query_sensor_data(query);
        match result {
            Ok(_) => assert!(false, "Unexpected result"),
            Err(e) => match e {
                Error::NotFound => assert!(true),
                _ => assert!(false, "Unexpected result"),
            },
        }
    }

    #[test]
    fn test_insert_scd4xdata_then_delete() {
        let now = chrono::Utc::now().naive_utc();

        let new_data = models::NewSensorData {
            sensor: VALID_SCD41_API_ID.into(),
            serialized_data: "{\"co2\": 420, \"temperature\": 21.5, \"humidity\": 40.2}",
            added_at: now, // Example timestamp
        };

        save_new_sensor_data(new_data).expect("Data should be inserted correctly");

        let mut db_conn = get_db_pool().expect("Failed to get database connection");

        use crate::schema::sensor_data::dsl::*;
        let count = diesel::delete(sensor_data.filter(added_at.eq(now)))
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
        let new_data = models::NewSensorData {
            sensor: "94a990533d762222222222222222222222222222e".into(),
            serialized_data: "{\"co2\": 420, \"temperature\": 21.5, \"humidity\": 40.2}",
            added_at: chrono::Utc::now().naive_utc(),
        };

        let result = save_new_sensor_data(new_data);
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
            user_api_id: ApiId::from_string(VALID_USER_API_ID).expect("Valid API ID"),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };

        let result =
            query_sensor_data(query).expect("Failed to query SCD4X data for existing user");
        assert_eq!(result.0, SensorKind::Scd4x,);
        assert!(
            result.1.len() == 10,
            "Expected 10 records, found {}",
            result.1.len()
        );
    }

    #[test]
    fn test_query_scd4x_data_nonexistent_user() {
        let query = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string("abcdef9999abcdef9999abcdef9999abcdef9999")
                .expect("Valid API ID"),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).expect("Valid API ID"),
            added_at_upper: None,
            added_at_lower: None,
        };
        let result = query_sensor_data(query);

        match result {
            Ok(_) => assert!(false, "Unexpected result"),
            Err(e) => match e {
                Error::NotFound => assert!(true),
                _ => assert!(false, "Unexpected result"),
            },
        }
    }

    // AI
    #[test]
    fn test_new_place_then_delete() {
        let user_api_id = ApiId::from_string(VALID_USER_API_ID).unwrap();
        let username = "testuser";
        let place_name = "Test Place";
        let place_description = Some("A place for testing");
        let color = PlaceColor::HEX_2A4039;

        // Create a new place
        let new_place_result =
            new_place(&user_api_id, username, place_name, place_description, color);
        assert!(
            new_place_result.is_ok(),
            "Failed to create new place: {:?}",
            new_place_result.err()
        );
        let place_api_id = new_place_result.unwrap();

        // Clean up by deleting the place
        let delete_result = delete_place(&user_api_id, &place_api_id);
        assert!(
            delete_result.is_ok(),
            "Failed to delete place: {:?}",
            delete_result.err()
        );
    }

    #[test]
    fn test_delete_place_unauthorized() {
        let unauthorized_user_api_id = ApiId::random();
        let place_api_id = ApiId::from_string(VALID_PLACE_ID_1).unwrap();

        let result = delete_place(&unauthorized_user_api_id, &place_api_id);
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn test_update_sensor_last_measurement() {
        let sensor_api_id = VALID_AHT10_API_ID;
        let new_time = chrono::Utc::now().naive_utc();

        // Update the last measurement
        let update_result = update_sensor_last_measurement(new_time, sensor_api_id);
        assert!(
            update_result.is_ok(),
            "Update failed: {:?}",
            update_result.err()
        );

        // Verify the update
        use crate::schema::user_sensors::dsl::*;
        let mut db_conn = get_db_pool().unwrap();
        let updated_time = user_sensors
            .find(sensor_api_id)
            .select(last_measurement)
            .first::<NaiveDateTime>(&mut db_conn)
            .unwrap();

        // Timestamps can have precision differences, so we check if they are very close
        assert!((updated_time - new_time).num_milliseconds() < 10);
    }

    #[test]
    fn test_update_sensor_last_measurement_not_found() {
        let sensor_api_id = "nonexistent-sensor-id";
        let new_time = chrono::Utc::now().naive_utc();
        let result = update_sensor_last_measurement(new_time, sensor_api_id);
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn test_get_sensor_kind_from_id() {
        let kind = get_sensor_kind_from_id(VALID_AHT10_API_ID).unwrap();
        assert_eq!(kind, SensorKind::Aht10);

        let kind = get_sensor_kind_from_id(VALID_SCD41_API_ID).unwrap();
        assert_eq!(kind, SensorKind::Scd4x);
    }

    #[test]
    fn test_get_sensor_kind_from_id_not_found() {
        let result = get_sensor_kind_from_id("nonexistent-sensor-id");
        assert!(matches!(
            result,
            Err(Error::DataBaseError(diesel::result::Error::NotFound))
        ));
    }

    #[test]
    fn test_query_sensor_data_with_date_range() {
        // Insert test data with specific timestamps
        let now = chrono::Utc::now();
        let data_points = [
            (now - chrono::Duration::days(4)).naive_utc(),
            (now - chrono::Duration::days(3)).naive_utc(),
            (now - chrono::Duration::days(1)).naive_utc(),
        ];

        for &ts in &data_points {
            let new_data = models::NewSensorData {
                sensor: VALID_AHT10_API_ID.into(),
                serialized_data: "{}",
                added_at: ts,
            };
            save_new_sensor_data(new_data).unwrap();
        }

        // Query for a specific date range
        let query = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            sensor_api_id: ApiId::from_string(VALID_AHT10_API_ID).unwrap(),
            added_at_lower: Some((now - chrono::Duration::days(5)).timestamp() as u32),
            added_at_upper: Some((now - chrono::Duration::days(2)).timestamp() as u32),
        };

        let result = query_sensor_data(query).unwrap();
        // Should find the points at T-3 days and T-2 days.
        assert_eq!(result.1.len(), 2);

        // Clean up inserted data
        use crate::schema::sensor_data::dsl::*;
        let mut db_conn = get_db_pool().unwrap();
        diesel::delete(sensor_data.filter(added_at.eq_any(&data_points)))
            .execute(&mut db_conn)
            .unwrap();
    }
}
