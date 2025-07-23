// place_colors: {
//     a: "#403E2A",
//     b: "#807895",
//     c: "#2A4039",
//     d: "#402E2A",
//     e: "#957E78",
//     f: "#302A40",
//     g: "#807E71",
//     h: "#78958B",
//     i: "#BFBA7A",
//     j: "#EA937D"
// }

// sensor_colors: {
//     a: "#DB2122",
//     b: "#F0D16F",
//     c: "#21DB55",
//     d: "#2132DB",
//     e: "#6FF0D1",
//     f: "#DB21A0",
//     g: "#DB8F21",
// }

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, Debug, Clone, TS, Eq, PartialEq, Hash)]
#[ts(export)]
#[allow(non_camel_case_types)]
pub enum PlaceColor {
    HEX_403E2A,
    HEX_807895,
    HEX_2A4039,
    HEX_402E2A,
    HEX_957E78,
    HEX_302A40,
    HEX_807E71,
    HEX_78958B,
    HEX_BFBA7A,
    HEX_EA937D,
}

impl PlaceColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlaceColor::HEX_403E2A => "#403E2A",
            PlaceColor::HEX_807895 => "#807895",
            PlaceColor::HEX_2A4039 => "#2A4039",
            PlaceColor::HEX_402E2A => "#402E2A",
            PlaceColor::HEX_957E78 => "#957E78",
            PlaceColor::HEX_302A40 => "#302A40",
            PlaceColor::HEX_807E71 => "#807E71",
            PlaceColor::HEX_78958B => "#78958B",
            PlaceColor::HEX_BFBA7A => "#BFBA7A",
            PlaceColor::HEX_EA937D => "#EA937D",
        }
    }

    pub fn from_str(color: &str) -> Option<PlaceColor> {
        match color {
            "#403E2A" => Some(PlaceColor::HEX_403E2A),
            "#807895" => Some(PlaceColor::HEX_807895),
            "#2A4039" => Some(PlaceColor::HEX_2A4039),
            "#402E2A" => Some(PlaceColor::HEX_402E2A),
            "#957E78" => Some(PlaceColor::HEX_957E78),
            "#302A40" => Some(PlaceColor::HEX_302A40),
            "#807E71" => Some(PlaceColor::HEX_807E71),
            "#78958B" => Some(PlaceColor::HEX_78958B),
            "#BFBA7A" => Some(PlaceColor::HEX_BFBA7A),
            "#EA937D" => Some(PlaceColor::HEX_EA937D),
            _ => None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, TS, Eq, PartialEq, Hash)]
#[ts(export)]
#[allow(non_camel_case_types)]
pub enum SensorColor {
    HEX_DB2122,
    HEX_F0D16F,
    HEX_21DB55,
    HEX_2132DB,
    HEX_6FF0D1,
    HEX_DB21A0,
    HEX_DB8F21,
}

impl SensorColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            SensorColor::HEX_DB2122 => "#DB2122",
            SensorColor::HEX_F0D16F => "#F0D16F",
            SensorColor::HEX_21DB55 => "#21DB55",
            SensorColor::HEX_2132DB => "#2132DB",
            SensorColor::HEX_6FF0D1 => "#6FF0D1",
            SensorColor::HEX_DB21A0 => "#DB21A0",
            SensorColor::HEX_DB8F21 => "#DB8F21",
        }
    }

    pub fn from_str(color: &str) -> Option<SensorColor> {
        match color {
            "#DB2122" => Some(SensorColor::HEX_DB2122),
            "#F0D16F" => Some(SensorColor::HEX_F0D16F),
            "#21DB55" => Some(SensorColor::HEX_21DB55),
            "#2132DB" => Some(SensorColor::HEX_2132DB),
            "#6FF0D1" => Some(SensorColor::HEX_6FF0D1),
            "#DB21A0" => Some(SensorColor::HEX_DB21A0),
            "#DB8F21" => Some(SensorColor::HEX_DB8F21),
            _ => None,
        }
    }
}
