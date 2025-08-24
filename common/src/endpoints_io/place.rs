use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

use crate::types::{
    ApiTimestamp,
    validate::{
        api_color::ApiColor, api_description::ApiDescription, api_entity_name::ApiEntityName,
    },
};

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, serde_valid::Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub struct ApiUserPlace {
    #[validate]
    pub name: ApiEntityName,
    #[validate]
    pub description: Option<ApiDescription>,
    #[validate]
    pub color: ApiColor,
    pub created_at: ApiTimestamp,
    pub updated_at: ApiTimestamp,
}

// impl ApiUserPlace {
//     pub fn from_user_place_and_color(place: UserPlace, color: String) -> Self {
//         Self {
//             name: place.name.into(),
//             description: place.description.map(|d| d.into()),
//             color: color.into(),
//             created_at: place.created_at.and_utc().timestamp() as ApiTimestamp,
//             updated_at: place.updated_at.and_utc().timestamp() as ApiTimestamp,
//         }
//     }
// }

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Clone, Validate)]
pub enum PlaceChange {
    Name(#[validate] ApiEntityName),
    Description(#[validate] Option<ApiDescription>),
    Color(#[validate] ApiColor),
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct PutPlace {
    pub place_name: ApiEntityName,
    #[validate]
    pub change: PlaceChange,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
pub enum GetPlaceEnum {
    FromPlaceName(#[validate] ApiEntityName),
    UserPlaces,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub struct GetPlace {
    #[serde(flatten)]
    #[validate]
    pub param: GetPlaceEnum,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub enum DeletePlace {
    FromPlaceName(#[validate] ApiEntityName),
    UserPlaces,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone, Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub struct PostPlace {
    #[validate]
    pub name: ApiEntityName,
    #[validate]
    pub description: Option<ApiDescription>,
    #[validate]
    pub color: ApiColor,
}
