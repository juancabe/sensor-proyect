use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::aht10data)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Aht10Data {
    pub data_id: i32,
    pub user_uuid: String,
    pub user_place_id: i32,
    pub serialized_data: String,
    pub added_at: i64,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::aht10data)]
pub struct NewAht10Data<'a> {
    pub user_uuid: &'a str,
    pub user_place_id: i32,
    pub serialized_data: &'a str,
    pub added_at: i64, // UNIX timestamp in seconds
}

// #[derive(Queryable, Selectable, Clone)]
// #[diesel(table_name = crate::schema::scd4xdata)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct Scd4xData {
//     pub data_id: i32,
//     pub user_uuid: String,
//     pub user_place_id: i32,
//     pub serialized_data: String,
//     pub added_at: i64,
// }

// #[derive(Insertable, Clone, Debug)]
// #[diesel(table_name = crate::schema::scd4xdata)]
// pub struct NewScd4xData<'a> {
//     pub user_uuid: &'a str,
//     pub user_place_id: i32,
//     pub serialized_data: &'a str,
//     pub added_at: i64, // UNIX timestamp in seconds
// }

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_places)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserPlace {
    pub user_place_id: i32,
    pub user_id: String,
    pub place_name: String,
    pub place_description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::user_places)]
pub struct NewUserPlace<'a> {
    pub user_id: &'a str,
    pub place_name: &'a str,
    pub place_description: Option<&'a str>,
    pub created_at: i64, // UNIX timestamp in seconds
    pub updated_at: i64, // UNIX timestamp in seconds
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub uuid: String,
    pub username: String,
    pub hashed_password: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub uuid: &'a str, // UUID of the user
    pub username: &'a str,
    pub hashed_password: &'a str,
    pub email: &'a str,
    pub created_at: i64,
    pub updated_at: i64,
}
