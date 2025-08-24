use crate::db::{DbConn, Error, model, model::HexValue};
use diesel::prelude::*;

#[derive(Debug, Clone)]
pub enum Identifier {
    Id(i32),
    Hex(HexValue),
}

pub fn get_color_id(conn: &mut DbConn, identifier: Identifier) -> Result<i32, Error> {
    use crate::db::schema::{colors::dsl as color, colors::dsl::colors as colors_table};

    let r = match identifier {
        Identifier::Id(id) => colors_table
            .filter(color::id.eq(id))
            .select(color::id)
            .first::<i32>(conn)?,
        Identifier::Hex(color) => colors_table
            .filter(color::hex_value.eq(color))
            .select(color::id)
            .first::<i32>(conn)?,
    };

    Ok(r)
}

pub fn get_color_by_id(conn: &mut DbConn, id: i32) -> Result<String, Error> {
    use crate::db::schema::{colors::dsl as color, colors::dsl::colors as colors_table};

    let r = colors_table
        .filter(color::id.eq(id))
        .first::<model::Color>(conn)?;

    Ok(r.hex_value)
}

#[cfg(test)]
mod test {

    use common::types::validate::api_color::COLOR_HEX_STRS;

    use crate::db::establish_connection;

    use super::*;

    #[test]
    fn test_get_color_id() {
        let mut conn = establish_connection(true).expect("Should be available");

        for id in 1..10 {
            let identifier = Identifier::Id(id);
            let res = get_color_id(&mut conn, identifier).expect("Color should exist");
            assert_eq!(res, id);
        }

        for (index, color) in COLOR_HEX_STRS.iter().enumerate() {
            let identifier = Identifier::Hex(color.to_string());
            let res = get_color_id(&mut conn, identifier).expect("Color should exist");
            assert_eq!(index + 1, res as usize)
        }

        let res = [
            get_color_id(&mut conn, Identifier::Id(132231)),
            get_color_id(&mut conn, Identifier::Hex("#FF1234AA".to_string())),
            get_color_id(&mut conn, Identifier::Hex("ADWADA".to_string())),
        ];
        assert!(res.iter().all(|r| r.is_err()))
    }

    #[test]
    fn test_get_color_by_id() {
        let mut conn = establish_connection(true).expect("Should be available");

        for (index, color) in COLOR_HEX_STRS.iter().enumerate() {
            let res = get_color_by_id(&mut conn, (index + 1) as i32).expect("Color should exist");
            assert_eq!(res, *color);
        }

        let res = [
            get_color_by_id(&mut conn, 132231),
            get_color_by_id(&mut conn, 2311),
            get_color_by_id(&mut conn, -1),
            get_color_by_id(&mut conn, 1234),
        ];
        assert!(res.iter().all(|r| r.is_err()))
    }
}
