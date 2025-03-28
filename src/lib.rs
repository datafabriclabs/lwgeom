mod error;
mod foreign_type;
mod gbox;
mod lwcollection;
mod lwgeom;
mod lwgeom_parser_result;
mod lwpoly;
#[cfg(feature = "mvt")]
mod mvt;

pub use error::{LWGeomError, Result};
pub use gbox::GBox;
pub use lwgeom::{LWGeom, LWGeomRef};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        let result = LWGeom::from_ewkb(
            LWGeom::from_text(
                "MULTILINESTRING((10 10,190 190), (15 15,30 30,100 90))",
                None,
            )
            .unwrap()
            .split(&LWGeom::from_text("POINT(30 30)", None).unwrap())
            .as_ewkb()
            .unwrap()
            .as_slice(),
        )
        .unwrap()
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
            .calculate_bbox()
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
        let result = LWGeom::from_ewkb(
            LWGeom::tile_envelope(2, 1, 1, None, None)
                .unwrap()
                .as_ewkb()
                .unwrap()
                .as_slice(),
        )
        .unwrap()
        .as_text(None)
        .unwrap();
        assert_eq!(
            result,
            "POLYGON((-10018754.1713945 0,-10018754.1713945 10018754.1713945,0 10018754.1713945,0 0,-10018754.1713945 0))"
        );
    }

    #[cfg(feature = "mvt")]
    #[test]
    fn test_into_mvt_geom() {
        let result = LWGeom::from_ewkb(
            LWGeom::from_ewkt("POLYGON((0 0,10 0,10 5,0 -5,0 0))")
                .unwrap()
                .into_mvt_geom(
                    &GBox::make_box((0.0, 0.0), (4096.0, 4096.0)),
                    4096,
                    0,
                    false,
                )
                .unwrap()
                .as_ewkb()
                .unwrap()
                .as_slice(),
        )
        .unwrap()
        .as_text(None)
        .unwrap();
        assert_eq!(
            result,
            "MULTIPOLYGON(((5 4096,10 4091,10 4096,5 4096)),((5 4096,0 4101,0 4096,5 4096)))"
        );
    }
}
