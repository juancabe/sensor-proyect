use crate::db::{DbConn, Error};
use diesel::prelude::*;

#[derive(Debug)]
pub enum Identifier {
    Id(i32),
    Username(String),
}

pub fn get_user_id(conn: &mut DbConn, identifier: Identifier) -> Result<i32, Error> {
    use crate::schema::{users::dsl as user, users::dsl::users as users_table};

    let r = match identifier {
        Identifier::Id(id) => users_table
            .filter(user::id.eq(id))
            .select(user::id)
            .first::<i32>(conn)?,
        Identifier::Username(username) => users_table
            .filter(user::username.eq(username))
            .select(user::id)
            .first::<i32>(conn)?,
    };
    Ok(r)
}

pub fn get_user_password(conn: &mut DbConn, identifier: Identifier) -> Result<String, Error> {
    use crate::schema::{users::dsl as user, users::dsl::users as users_table};

    let r = match identifier {
        Identifier::Id(id) => users_table
            .filter(user::id.eq(id))
            .select(user::hashed_password)
            .first::<String>(conn)?,
        Identifier::Username(username) => users_table
            .filter(user::username.eq(username))
            .select(user::hashed_password)
            .first::<String>(conn)?,
    };
    Ok(r)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::db::{establish_connection, tests::create_test_user};

    #[test]
    fn test_get_user_id() {
        let mut conn = establish_connection().expect("Correct!!");

        let user = create_test_user(&mut conn);

        let identifier = Identifier::Username(user.username);
        let res1 = get_user_id(&mut conn, identifier).expect("Should work!");

        assert!(res1 == user.id);
    }
}
