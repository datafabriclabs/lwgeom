mod error;
mod gbox;
mod lwgeom;
mod lwgeom_parser_result;
mod lwpoly;

pub use error::{LWGeomError, Result};
pub use gbox::{GBox, GBoxRef};
pub use lwgeom::{LWGeom, LWGeomRef};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        let result = LWGeom::from_text(
            "MULTILINESTRING((10 10,190 190), (15 15,30 30,100 90))",
            None,
        )
        .unwrap()
        .split(&LWGeom::from_text("POINT(30 30)", None).unwrap())
        .as_text(None)
        .unwrap();
        assert_eq!(
            result,
            "GEOMETRYCOLLECTION(LINESTRING(10 10,30 30),LINESTRING(30 30,190 190),LINESTRING(15 15,30 30),LINESTRING(30 30,100 90))"
        );
    }

    #[test]
    fn test_box2d() {
        let result = LWGeom::from_text("LINESTRING(1 2, 3 4, 5 6)", None)
            .unwrap()
            .get_bbox_ref()
            .to_string();
        assert_eq!(result, "GBOX((1,2),(5,6))");
    }

    #[test]
    fn test_from_ewkb() {
        let result = LWGeom::from_ewkb(
            &LWGeom::from_text("LINESTRING(1 2, 3 4, 5 6)", None)
                .unwrap()
                .as_ewkb()
                .unwrap(),
        )
        .unwrap()
        .as_ewkt(None)
        .unwrap();
        assert_eq!(result, "LINESTRING(1 2,3 4,5 6)");
    }

    #[test]
    fn test_tile_envelope() {
        let result = LWGeom::tile_envelope(2, 1, 1, None, None)
            .unwrap()
            .as_text(None)
            .unwrap();
        assert_eq!(
            result,
            "POLYGON((-10018754.1713945 0,-10018754.1713945 10018754.1713945,0 10018754.1713945,0 0,-10018754.1713945 0))"
        );
    }
}
