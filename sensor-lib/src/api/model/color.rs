// INSERT INTO colors (hex_value, name) VALUES
// ('#FF0000', 'Red'),
// ('#0000FF', 'Blue'),
// ('#FFFF00', 'Yellow'),
// ('#008000', 'Green'),
// ('#FFA500', 'Orange'),
// ('#800080', 'Purple'),
// ('#FFFFFF', 'White'),
// ('#000000', 'Black'),
// ('#808080', 'Gray');

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, Debug, Clone, TS, Eq, PartialEq, Hash)]
#[ts(export)]
#[allow(non_camel_case_types)]
pub enum Color {
    Red_FF0000,
    Blue_0000FF,
    Yellow_FFFF00,
    Green_008000,
    Orange_FFA500,
    Purple_800080,
    White_FFFFFF,
    Black_000000,
    Gray_808080,
}

impl Color {
    /// Converts a string slice to a `Color` enum variant.
    ///
    /// This function is case-sensitive. It returns an error if the string
    /// does not match any of the defined color variants.
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        match s {
            "Red_FF0000" => Ok(Color::Red_FF0000),
            "Blue_0000FF" => Ok(Color::Blue_0000FF),
            "Yellow_FFFF00" => Ok(Color::Yellow_FFFF00),
            "Green_008000" => Ok(Color::Green_008000),
            "Orange_FFA500" => Ok(Color::Orange_FFA500),
            "Purple_800080" => Ok(Color::Purple_800080),
            "White_FFFFFF" => Ok(Color::White_FFFFFF),
            "Black_000000" => Ok(Color::Black_000000),
            "Gray_808080" => Ok(Color::Gray_808080),
            _ => Err("Invalid color string"),
        }
    }

    /// Returns a string slice representation of the `Color` enum variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            Color::Red_FF0000 => "Red_FF0000",
            Color::Blue_0000FF => "Blue_0000FF",
            Color::Yellow_FFFF00 => "Yellow_FFFF00",
            Color::Green_008000 => "Green_008000",
            Color::Orange_FFA500 => "Orange_FFA500",
            Color::Purple_800080 => "Purple_800080",
            Color::White_FFFFFF => "White_FFFFFF",
            Color::Black_000000 => "Black_000000",
            Color::Gray_808080 => "Gray_808080",
        }
    }

    pub fn hex_value(&self) -> String {
        return "#".to_string()
            + self
                .as_str()
                .split("_")
                .last()
                .expect("all colors should have a '_'");
    }
}
