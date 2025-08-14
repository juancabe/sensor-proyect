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
