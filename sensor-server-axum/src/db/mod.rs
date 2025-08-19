pub mod colors;
pub mod sensor_data;
pub mod user_places;
pub mod user_sensors;
pub mod users;

use dotenv::dotenv;
use hyper::StatusCode;
use r2d2::{self as original_r2d2, PooledConnection};

use diesel::{Connection, PgConnection, r2d2::ConnectionManager, result::DatabaseErrorKind};
use std::{fmt::Display, sync::LazyLock};

pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;
// pub type DbConn = PgConnection;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbConnHolder(pub DbConn);

static DB_POOL: LazyLock<DbPool> = LazyLock::new(|| {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<diesel::PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
});

type ExternalError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Error {
    ConnectionError(ExternalError),
    NotFound(ExternalError),
    InternalError(ExternalError),
    NotUnique(ExternalError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionError(error) => write!(f, "Connection Error: {error:?}"),
            Error::NotFound(error) => write!(f, "Not Found: {error:?}"),
            Error::InternalError(error) => write!(f, "Internal Error: {error:?}"),
            Error::NotUnique(error) => write!(f, "NotUnique Error: {error:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for StatusCode {
    fn from(err: Error) -> Self {
        match err {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::ConnectionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotUnique(_) => StatusCode::CONFLICT,
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => {
                log::warn!("Diesel NotFound will translate into NotFound: {value:?} ");
                Self::NotFound(value.into())
            }
            e => match e {
                diesel::result::Error::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    err_info,
                ) => {
                    log::warn!(
                        "Diesel UniqueViolation will translate into NotUnique: {err_info:?} "
                    );
                    Self::NotUnique("NotUnique".into())
                }
                _ => {
                    log::error!("Diesel error will return InternalError: {e:?}");
                    Self::InternalError(e.into())
                }
            },
        }
    }
}

impl From<original_r2d2::Error> for Error {
    fn from(value: original_r2d2::Error) -> Self {
        log::error!("r2d2 error will return ConnectionError: {value:?}");
        Self::ConnectionError(value.into())
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(value: diesel::ConnectionError) -> Self {
        log::error!("diesel::ConnectionError error will return ConnectionError: {value:?}");
        Self::ConnectionError(value.into())
    }
}

pub fn establish_connection(test: bool) -> Result<DbConn, Error> {
    dotenv().expect(".env should be available and readable");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    assert_eq!(database_url.contains("test_database"), test);

    // let mut conn: DbConn = PgConnection::establish(&database_url)?;
    let mut conn;
    if test {
        let manager = ConnectionManager::<diesel::PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        conn = pool.get()?;
        conn.begin_test_transaction().expect("Valid");
        log::warn!("TEST DATABASE CREATED");
    } else {
        conn = DB_POOL.get()?;
    }

    Ok(conn)
}

#[cfg(test)]
pub mod tests {

    use std::ops::Range;

    use diesel::{Insertable, RunQueryDsl};
    use rand::{Rng, distr::Alphabetic};
    use sensor_lib::api::model::api_id::ApiId;

    use crate::{
        db::DbConn,
        model::{NewUser, NewUserPlace, NewUserSensor, User, UserPlace, UserSensor},
    };

    pub fn random_string(range: Range<usize>) -> String {
        rand::rng()
            .sample_iter(&Alphabetic)
            .take(rand::random_range(range))
            .map(char::from)
            .collect()
    }

    pub fn create_test_user(conn: &mut DbConn) -> User {
        use crate::schema::users::dsl::users as users_table;

        let username = random_string(5..16);
        let email = random_string(5..16);

        let test_user = NewUser {
            username,
            hashed_password: "hashed_password".to_string(),
            email,
        };

        let res: Vec<User> = test_user
            .clone()
            .insert_into(users_table)
            .load(conn)
            .expect("Should be insertable");

        assert_eq!(res.len(), 1);

        res.first().expect("Shopuld exist").clone()
    }

    pub fn create_test_user_place(conn: &mut DbConn, user: &User) -> UserPlace {
        use crate::schema::user_places::dsl::user_places as user_places_table;

        let name = random_string(5..16);
        let description = random_string(5..16);

        let test_user_place = NewUserPlace {
            user_id: user.id,
            name,
            description: Some(description),
            color_id: 1,
        };

        let res: Vec<UserPlace> = test_user_place
            .clone()
            .insert_into(user_places_table)
            .load(conn)
            .expect("Should be insertable");

        assert_eq!(res.len(), 1);

        res.first().expect("Should exist").clone()
    }

    pub fn create_test_user_sensor(conn: &mut DbConn, user_place: &UserPlace) -> UserSensor {
        use crate::schema::user_sensors::dsl::user_sensors as user_sensors_table;

        let name = random_string(5..16);
        let description = random_string(5..16);

        let test_user_sensor = NewUserSensor {
            name,
            description: Some(description),
            color_id: 1,
            place_id: user_place.id,
            device_id: ApiId::random().to_string(),
        };

        let res: Vec<UserSensor> = test_user_sensor
            .clone()
            .insert_into(user_sensors_table)
            .load(conn)
            .expect("Should be insertable");

        assert_eq!(res.len(), 1);

        res.first().expect("Should exist").clone()
    }
}
