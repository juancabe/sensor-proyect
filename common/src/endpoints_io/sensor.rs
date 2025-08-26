use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

use crate::{
    endpoints_io::sensor_data::ApiSensorData,
    types::{
        ApiTimestamp,
        validate::{
            api_color::ApiColor, api_description::ApiDescription, api_entity_name::ApiEntityName,
            api_pub_key::ApiPubKey, device_id::DeviceId,
        },
    },
};

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct ApiUserSensor {
    #[validate]
    pub device_id: DeviceId,
    #[validate]
    pub pub_key: ApiPubKey,
    #[validate]
    pub name: ApiEntityName,
    #[validate]
    pub description: Option<ApiDescription>,
    #[validate]
    pub color: ApiColor,
    pub created_at: ApiTimestamp,
    pub updated_at: ApiTimestamp,
    pub place_name: ApiEntityName,
}

// impl ApiUserSensor {
//     pub fn from_sensor_place_color(
//         sensor: UserSensor,
//         color: String,
//         place_name: String,
//     ) -> Result<Self, device_id::Error> {
//         Ok(Self {
//             device_id: DeviceId::from_string(&sensor.device_id)?,
//             name: sensor.name.into(),
//             description: sensor.description.map(|d| d.into()),
//             color: ApiColor::from(color),
//             created_at: sensor.created_at.and_utc().timestamp() as ApiTimestamp,
//             updated_at: sensor.updated_at.and_utc().timestamp() as ApiTimestamp,
//             place_name: place_name.into(),
//         })
//     }
// }

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
pub enum GetSensorEnum {
    FromSensorDeviceId(DeviceId),
    FromPlaceName(#[validate] ApiEntityName),
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct GetSensor {
    #[serde(flatten)]
    #[validate]
    pub param: GetSensorEnum,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct GetSensorResponse {
    pub sensor: ApiUserSensor,
    pub last_data: Option<ApiSensorData>,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Clone, Validate)]
pub enum SensorChange {
    PlaceName(#[validate] ApiEntityName),
    Name(#[validate] ApiEntityName),
    Description(#[validate] Option<ApiDescription>),
    Color(#[validate] ApiColor),
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct PutSensor {
    pub device_id: DeviceId,
    #[validate]
    pub change: SensorChange,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Clone, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct PostSensor {
    #[validate]
    pub place_name: ApiEntityName,
    #[validate]
    pub device_id: DeviceId,
    #[validate]
    pub pub_key: ApiPubKey,
    #[validate]
    pub name: ApiEntityName,
    #[validate]
    pub description: Option<ApiDescription>,
    #[validate]
    pub color: ApiColor,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub enum DeleteSensor {
    FromSensorDeviceId(DeviceId),
    FromPlaceName(#[validate] ApiEntityName),
}
