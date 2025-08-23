use std::ops::Range;

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::{
    db::{DbConn, Error},
    db::model::{NewSensorData, SensorData},
};

pub fn insert_sensor_data(conn: &mut DbConn, new_data: NewSensorData) -> Result<SensorData, Error> {
    use crate::db::schema::sensor_data::dsl::sensor_data as sensor_data_table;

    let data: Vec<SensorData> = new_data.insert_into(sensor_data_table).load(conn)?;

    let data = data
        .into_iter()
        .next()
        .ok_or(Error::NotFound("The returned vec was empty".into()))
        .map(|e| e.clone())?;

    log::trace!("Data added: {data:?}");
    Ok(data)
}

pub enum Identifier {
    SensorId(i32),
}

pub fn get_sensor_data(
    conn: &mut DbConn,
    identifier: Identifier,
    range: Range<NaiveDateTime>,
) -> Result<Vec<SensorData>, Error> {
    match identifier {
        Identifier::SensorId(device_id) => {
            use crate::db::schema::{
                sensor_data::dsl as sensor_data, sensor_data::dsl::sensor_data as sensor_data_table,
            };

            let res = sensor_data_table
                .filter(sensor_data::sensor_id.eq(device_id))
                .filter(sensor_data::added_at.between(range.start, range.end))
                .load(conn)?;

            log::trace!("DB Returned {} items", res.len());

            Ok(res)
        }
    }
}
