use crate::{
    db::{DbConn, Error},
    model::{self, HexValue},
};
use diesel::prelude::*;

pub enum Identifier {
    Id(i32),
    Hex(HexValue),
}

pub fn get_color_id(conn: &mut DbConn, identifier: Identifier) -> Result<i32, Error> {
    use crate::schema::{colors::dsl as color, colors::dsl::colors as colors_table};

    let r = match identifier {
        Identifier::Id(id) => id,
        Identifier::Hex(color) => colors_table
            .filter(color::hex_value.eq(color))
            .select(color::id)
            .first::<i32>(conn)?,
    };
    Ok(r)
}

pub fn get_color_by_id(conn: &mut DbConn, id: i32) -> Result<String, Error> {
    use crate::schema::{colors::dsl as color, colors::dsl::colors as colors_table};

    let r = colors_table
        .filter(color::id.eq(id))
        .first::<model::Color>(conn)?;

    Ok(r.hex_value)
}
