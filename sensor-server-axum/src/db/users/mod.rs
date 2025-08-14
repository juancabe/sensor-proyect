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
