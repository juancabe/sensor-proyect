pub mod user_places;

use r2d2 as original_r2d2;

use diesel::{PgConnection, r2d2::ConnectionManager};
use std::{fmt::Display, sync::LazyLock};

pub type DbPool = r2d2::Pool<ConnectionManager<diesel::PgConnection>>;
pub type DbConn = PgConnection;

pub static DB_POOL: LazyLock<DbPool> = LazyLock::new(|| {
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
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConnectionError(error) => write!(f, "Connection Error: {error:?}"),
            Error::NotFound(error) => write!(f, "Not Found: {error:?}"),
            Error::InternalError(error) => write!(f, "Internal Error: {error:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<diesel::result::Error> for Error {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::NotFound(value.into()),
            _ => Self::InternalError(value.into()),
        }
    }
}

impl From<original_r2d2::Error> for Error {
    fn from(value: original_r2d2::Error) -> Self {
        Self::ConnectionError(value.into())
    }
}

#[cfg(test)]
pub mod tests {

    use diesel::{Connection, Insertable, PgConnection, RunQueryDsl};
    use dotenv::dotenv;
    use sensor_lib::api::model::api_id::ApiId;

    use crate::{
        db::DbConn,
        model::{NewUser, NewUserPlace, User, UserPlace},
    };

    pub fn establish_test_connection() -> PgConnection {
        dotenv().expect(".env should be available and readable");

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

        if !database_url.contains("test_database") {
            panic!("We are not using the test database");
        }

        let mut conn = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));
        conn.begin_test_transaction().expect("Valid");
        conn
    }

    pub fn create_test_user(conn: &mut DbConn) -> User {
        use crate::schema::users::dsl::users as users_table;

        let test_user = NewUser {
            username: "testuser".to_string(),
            api_id: ApiId::random().to_string(),
            hashed_password: "hashed_password".to_string(),
            email: "testuser@email.com".to_string(),
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

        let test_user_place = NewUserPlace {
            api_id: ApiId::random().to_string(),
            user_id: user.id,
            name: "testuserplace".to_string(),
            description: Some("le description".to_string()),
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
}
