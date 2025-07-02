use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;

use once_cell::sync::Lazy;
use sensor_lib::api;
use sensor_lib::api::endpoints::get_aht10_data::GetAht10RequestBody;

use crate::models;

use diesel::r2d2::Error as DieselPoolError;

type FailedDeserialize = usize;

#[derive(Debug)]
pub enum Error {
    DataBaseError(diesel::result::Error),
    DataBaseConnectionError,
    SerializationError(serde_json::Error),
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
    query: GetAht10RequestBody,
) -> Result<(Vec<api::model::aht10_data::Aht10Data>, FailedDeserialize), Error> {
    use crate::schema::aht10data::dsl::*;

    let mut db_conn = get_db_pool()?;

    let max_date = query.added_at_upper;
    let min_date = query.added_at_lower;
    let u_uuid = &query.user_uuid;
    let u_place_id = query.user_place_id;

    let mut failed_deserialize = 0;

    let vec = aht10data
        .filter(user_uuid.eq(u_uuid))
        .filter(user_place_id.eq(u_place_id))
        .filter(added_at.le(max_date.unwrap_or(i64::MAX)))
        .filter(added_at.ge(min_date.unwrap_or(0)))
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

pub fn save_new_aht10_data(data: models::NewAht10Data<'_>) -> Result<(), Error> {
    use crate::schema::aht10data;

    let mut db_conn: r2d2::PooledConnection<ConnectionManager<SqliteConnection>> = get_db_pool()?;

    diesel::insert_into(aht10data::table)
        .values(data)
        .execute(&mut db_conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

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
    #[should_panic(expected = "Failed to insert mock user")]
    fn test_load_mock_data() {
        let mock_user = models::NewUser {
            uuid: "test-user-uuid",
            username: "testuser",
            hashed_password: "hashedpassword",
            email: "testuser@example.com",
            created_at: 1633036800, // Example timestamp
            updated_at: 1633036800, // Example timestamp
        };

        let mock_user_place = models::NewUserPlace {
            user_id: mock_user.uuid,
            place_name: "Test Place",
            place_description: Some("A test place for unit tests"),
            created_at: 1633036800, // Example timestamp
            updated_at: 1633036800, // Example timestamp
        };

        const DATA_COUNT: usize = 100;
        let mut serialized_data = api::model::aht10_data::Aht10Data {
            temperature: 22.5,
            humidity: 45.0,
            sensor_id: "test-sensor-id".to_string(),
        };

        let mut model_data = models::NewAht10Data {
            user_uuid: &mock_user.uuid,
            user_place_id: 1,
            serialized_data: &serde_json::to_string(&serialized_data).unwrap(),
            added_at: 1633036800, // Example timestamp
        };

        use rand::Rng;

        let mut db_conn = get_db_pool().expect("Failed to get database connection");

        // Insert mock user
        diesel::insert_into(crate::schema::users::table)
            .values(&mock_user)
            .execute(&mut db_conn)
            .expect("Failed to insert mock user");

        // Insert mock user place
        diesel::insert_into(crate::schema::user_places::table)
            .values(&mock_user_place)
            .execute(&mut db_conn)
            .expect("Failed to insert mock user place");

        let mut serialized_vec = Vec::with_capacity(DATA_COUNT);

        for _ in 0..DATA_COUNT {
            let serialized = serde_json::to_string(&serialized_data).unwrap();
            serialized_vec.push(serialized);
        }

        for i in 0..DATA_COUNT {
            let serialized_ref = &serialized_vec[i];

            serialized_data.temperature += rand::rng().random_range(-1.0..1.0);
            serialized_data.humidity += rand::rng().random_range(-1.0..1.0);
            model_data.serialized_data = serialized_ref;
            model_data.added_at = 1633036800 + (i as i64 * 60); // Increment timestamp by 60 seconds

            diesel::insert_into(crate::schema::aht10data::table)
                .values(&model_data)
                .execute(&mut db_conn)
                .expect("Failed to insert mock AHT10 data");
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
        let now = chrono::Utc::now().timestamp();

        let new_data = models::NewAht10Data {
            user_uuid: "test-user-uuid",
            user_place_id: 1,
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
    fn test_insert_aht10data_unexistent_user() {
        let new_data = models::NewAht10Data {
            user_uuid: "nonexistent",
            user_place_id: 1,
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
        let query = GetAht10RequestBody {
            user_uuid: "test-user-uuid".to_string(),
            user_place_id: 1,
            added_at_upper: None,
            added_at_lower: None,
        };

        let result = query_aht10_data(query).expect("Failed to query AHT10 data for existing user");
        assert!(
            result.0.len() == 100,
            "Expected 100 records, found {}",
            result.0.len()
        );
        assert_eq!(
            result.1, 0,
            "Expected no deserialization errors, found {}",
            result.1
        );
    }

    #[test]
    fn test_query_aht10_data_nonexistent_user() {
        let query = GetAht10RequestBody {
            user_uuid: "nonexistent".to_string(),
            user_place_id: 1,
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
}
