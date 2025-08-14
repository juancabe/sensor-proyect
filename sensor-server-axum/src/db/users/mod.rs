use crate::db::{DbConn, Error};
use diesel::prelude::*;
pub enum Identifier {
    Id(i32),
    Username(String),
    ApiId(String),
}

pub fn get_user_id(conn: &mut DbConn, identifier: Identifier) -> Result<i32, Error> {
    use crate::schema::{users::dsl as user, users::dsl::users as users_table};

    let r = match identifier {
        Identifier::Id(id) => id,
        Identifier::Username(username) => users_table
            .filter(user::username.eq(username))
            .select(user::id)
            .first::<i32>(conn)?,
        Identifier::ApiId(api_id) => users_table
            .filter(user::api_id.eq(api_id))
            .select(user::id)
            .first::<i32>(conn)?,
    };
    Ok(r)
}

#[cfg(test)]
mod test {
    use sensor_lib::api::model::api_id::ApiId;

    use super::*;
    use crate::db::{establish_connection, tests::create_test_user};

    #[test]
    fn test_get_user_id() {
        let mut conn = establish_connection().expect("Correct!!");

        let user = create_test_user(&mut conn);

        let identifier = Identifier::Username(user.username);
        let res1 = get_user_id(&mut conn, identifier).expect("Should work!");
        let identifier = Identifier::ApiId(user.api_id);
        let res2 = get_user_id(&mut conn, identifier).expect("Should work!");
        let identifier = Identifier::ApiId(ApiId::random().to_string());
        let res3 = get_user_id(&mut conn, identifier);

        assert!(res3.is_err());
        assert!(res1 == res2);
        assert!(res1 == user.id);
    }
}
