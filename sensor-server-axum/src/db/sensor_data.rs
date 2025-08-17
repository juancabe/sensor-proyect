use std::ops::Range;

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::{
    db::{DbConn, Error},
    model::{NewSensorData, SensorData},
};

pub fn insert_sensor_data(conn: &mut DbConn, new_data: NewSensorData) -> Result<SensorData, Error> {
    use crate::schema::sensor_data::dsl::sensor_data as sensor_data_table;

    let data: Vec<SensorData> = new_data.insert_into(sensor_data_table).load(conn)?;

    data.first()
        .ok_or(Error::NotFound("The returned vec was empty".into()))
        .map(|e| e.clone())
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
            use crate::schema::{
                sensor_data::dsl as sensor_data, sensor_data::dsl::sensor_data as sensor_data_table,
            };

            let res = sensor_data_table
                .filter(sensor_data::sensor_id.eq(device_id))
                .filter(sensor_data::added_at.between(range.start, range.end))
                .load(conn)?;

            Ok(res)
        }
    }
}
