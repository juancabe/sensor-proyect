use crate::{
    api::endpoints::user::PutUser,
    db::{DbConn, Error},
    db::model::{NewUser, User},
};
use diesel::prelude::*;

#[derive(Debug, Clone)]
pub enum Identifier<'a> {
    Id(i32),
    Username(&'a str),
    Email(&'a str),
}

pub fn insert_user(conn: &mut DbConn, new_user: NewUser) -> Result<User, Error> {
    use crate::db::schema::users::dsl::users as users_table;
    let vec = new_user.insert_into(users_table).load(conn)?;
    let user: User = vec
        .into_iter()
        .next()
        .ok_or(Error::NotFound("NotFound".into()))?;

    Ok(user)
}

pub fn get_user(conn: &mut DbConn, identifier: Identifier) -> Result<User, Error> {
    use crate::db::schema::{users::dsl as user, users::dsl::users as users_table};

    let r = match identifier {
        Identifier::Id(id) => users_table
            .filter(user::id.eq(id))
            .select(User::as_select())
            .first::<User>(conn)?,
        Identifier::Username(username) => users_table
            .filter(user::username.eq(username))
            .select(User::as_select())
            .first::<User>(conn)?,
        Identifier::Email(email) => users_table
            .filter(user::email.eq(email))
            .select(User::as_select())
            .first::<User>(conn)?,
    };
    Ok(r)
}

pub type Update = PutUser;

pub fn update_user(
    conn: &mut DbConn,
    identifier: Identifier,
    update: Update,
) -> Result<User, Error> {
    use crate::db::schema::{users::dsl as user, users::dsl::users as users_table};

    let mut db_user = get_user(conn, identifier)?;

    match update {
        Update::Username(un) => db_user.username = un.into(),
        Update::RawPassword(rp) => {
            db_user.hashed_password = rp
                .hash()
                .or(Err(Error::InternalError("Could not hash".into())))?
        }
        Update::Email(em) => db_user.email = em.into(),
    }

    let rows = diesel::update(users_table)
        .filter(user::id.eq(db_user.id))
        .set(&db_user)
        .execute(conn)?;

    if rows == 0 {
        Err(Error::NotFound(
            "Not found, update didn't affect any rows".into(),
        ))?
    }

    Ok(db_user)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::db::{establish_connection, tests::create_test_user};

    #[test]
    fn test_get_user() {
        let mut conn = establish_connection(true).expect("Correct!!");

        let (user, _) = create_test_user(&mut conn);

        let identifier = Identifier::Username(&user.username);
        let res1 = get_user(&mut conn, identifier).expect("Should work!");

        let identifier = Identifier::Id(user.id);
        let res2 = get_user(&mut conn, identifier).expect("Should work!");

        assert!(res1.id == user.id);
        assert!(res2.id == user.id);
    }
}
