use core::ffi::CStr;
use core::mem::MaybeUninit;
use std::ffi::CString;

use libc::{c_char, c_int};
use lwgeom_sys::*;

use crate::foreign_type::{ForeignType, ForeignTypeRef, Opaque};
use crate::lwcollection::LWCollectionRef;
use crate::lwgeom_parser_result::LWGeomParserResult;
use crate::lwpoly::LWPoly;
use crate::{GBox, LWGeomError, Result};

pub struct LWGeom(*mut LWGEOM);

impl ForeignType for LWGeom {
    type FFIType = LWGEOM;

    fn from_ptr(ptr: *mut Self::FFIType) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWGeom from a null pointer."
        );
        Self(ptr)
    }

    fn as_mut_ptr(&mut self) -> *mut Self::FFIType {
        self.0
    }

    fn as_ptr(&self) -> *const Self::FFIType {
        self.0.cast_const()
    }
}

unsafe impl Send for LWGeom {}
unsafe impl Sync for LWGeom {}

impl Drop for LWGeom {
    fn drop(&mut self) {
        unsafe { lwgeom_free(self.as_mut_ptr()) }
    }
}

pub struct LWGeomRef(Opaque);

impl ForeignTypeRef for LWGeomRef {
    type FFIType = LWGEOM;
}

unsafe impl Send for LWGeomRef {}
unsafe impl Sync for LWGeomRef {}

impl LWGeom {
    pub fn from_text(wkt: &str, srid: Option<i32>) -> Result<Self> {
        let c_wkt = CString::new(wkt)?;
        let p_parser_result = MaybeUninit::uninit().as_mut_ptr();
        let result = unsafe {
            lwgeom_parse_wkt(
                p_parser_result,
                c_wkt.as_ptr().cast_mut(),
                LW_PARSER_CHECK_ALL as c_int,
            )
        };
        let mut parser_result = LWGeomParserResult::from_ptr(p_parser_result);
        if result == LW_FAILURE as c_int {
            return Err(LWGeomError::WKTParseError(parser_result.message().ok_or(
                LWGeomError::FailedWithoutMessageError("lwgeom_parse_wkt".to_owned()),
            )?));
        }

        let mut geom = parser_result.take_geom();
        if geom.has_srid() {
            panic!("OGC WKT expected, EWKT provided - use from_ewkt() for this")
        }

        if let Some(srid) = srid {
            geom.set_srid(srid);
        }
        Ok(geom)
    }

    pub fn from_ewkt(wkt: &str) -> Result<Self> {
        let c_wkt = CString::new(wkt)?;
        let p_parser_result = MaybeUninit::uninit().as_mut_ptr();
        let result = unsafe {
            lwgeom_parse_wkt(
                p_parser_result,
                c_wkt.as_ptr().cast_mut(),
                LW_PARSER_CHECK_ALL as c_int,
            )
        };
        let mut parser_result = LWGeomParserResult::from_ptr(p_parser_result);
        if result == LW_FAILURE as c_int {
            return Err(LWGeomError::WKTParseError(parser_result.message().ok_or(
                LWGeomError::FailedWithoutMessageError("lwgeom_parse_wkt".to_owned()),
            )?));
        }

        Ok(parser_result.take_geom())
    }

    pub fn from_ewkb(ewkb: &[u8]) -> Result<Self> {
        let p_geom =
            unsafe { lwgeom_from_wkb(ewkb.as_ptr(), ewkb.len(), LW_PARSER_CHECK_ALL as c_char) };
        if p_geom.is_null() {
            return Err(LWGeomError::NullPtrError);
        }

        Ok(Self::from_ptr(p_geom))
    }
}

impl LWGeom {
    pub fn as_text(&self, precision: Option<i32>) -> Result<String> {
        let precision = precision.unwrap_or(15);
        let mut sz = MaybeUninit::uninit();
        let p_wkt =
            unsafe { lwgeom_to_wkt(self.as_ptr(), WKT_ISO as u8, precision, sz.as_mut_ptr()) };
        if p_wkt.is_null() {
            return Err(LWGeomError::NullPtrError);
        }

        let c_wkt = unsafe {
            CStr::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(
                p_wkt.cast(),
                sz.assume_init(),
            ))
        };
        let text = c_wkt.to_string_lossy().into_owned();
        unsafe {
            lwfree(p_wkt.cast());
        }
        Ok(text)
    }

    pub fn as_ewkt(&self, precision: Option<i32>) -> Result<String> {
        let precision = precision.unwrap_or(15);
        let mut sz = MaybeUninit::uninit();
        let p_wkt = unsafe {
            lwgeom_to_wkt(
                self.as_ptr(),
                WKT_EXTENDED as u8,
                precision,
                sz.as_mut_ptr(),
            )
        };
        if p_wkt.is_null() {
            return Err(LWGeomError::NullPtrError);
        }

        let c_wkt = unsafe {
            CStr::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(
                p_wkt.cast(),
                sz.assume_init(),
            ))
        };
        let ewkt = c_wkt.to_string_lossy().into_owned();
        unsafe {
            lwfree(p_wkt.cast());
        }
        Ok(ewkt)
    }

    pub fn as_ewkb(&self) -> Result<Vec<u8>> {
        let p_varlena = unsafe { lwgeom_to_wkb_varlena(self.as_ptr(), WKB_EXTENDED as u8) };
        if p_varlena.is_null() {
            return Err(LWGeomError::NullPtrError);
        }

        let ewkb_slice = unsafe {
            core::slice::from_raw_parts(
                (*p_varlena).data.as_ptr().cast(),
                (*p_varlena).size as usize,
            )
        };
        let ewkb = ewkb_slice.to_vec();
        unsafe {
            lwfree(p_varlena.cast());
        }
        Ok(ewkb)
    }
}

impl LWGeom {
    pub fn has_srid(&self) -> bool {
        unsafe { lwgeom_has_srid(self.as_ptr()) != 0 }
    }

    pub fn get_srid(&self) -> Option<i32> {
        if self.has_srid() {
            Some(unsafe { lwgeom_get_srid(self.as_ptr()) })
        } else {
            None
        }
    }

    pub fn set_srid(&mut self, srid: i32) {
        unsafe { lwgeom_set_srid(self.as_mut_ptr(), srid) }
    }

    pub fn split(&self, blade: &LWGeom) -> Self {
        let p_geom = unsafe { lwgeom_split(self.as_ptr(), blade.as_ptr()) };
        Self::from_ptr(p_geom)
    }

    pub fn calculate_bbox(&self) -> GBox {
        let mut bbox = GBox::new_bbox();
        unsafe { lwgeom_calculate_gbox(self.as_ptr(), bbox.as_mut_ptr()) };
        bbox
    }

    pub fn tile_envelope(
        zoom: i32, x: i32, y: i32, bounds: Option<&LWGeom>, margin: Option<f64>,
    ) -> Result<Self> {
        let bounds = match bounds {
            Some(bounds) => bounds,
            None => &Self::from_ewkt("SRID=3857;LINESTRING(-20037508.342789 -20037508.342789,20037508.342789 20037508.342789)").unwrap(),
        };
        let bbox = bounds.calculate_bbox();

        let srid = bounds.get_srid().unwrap_or(3857);

        let margin = margin.unwrap_or(0.0);
        if margin < -0.5 {
            return Err(LWGeomError::InvalidParameterError(
                "ST_TileEnvelope".to_owned(),
                "margin".to_owned(),
            ));
        }

        let bounds_width = bbox.xmax() - bbox.xmin();
        let bounds_height = bbox.ymax() - bbox.ymin();
        if bounds_width <= 0.0 || bounds_height <= 0.0 {
            return Err(LWGeomError::InvalidParameterError(
                "ST_TileEnvelope".to_owned(),
                "bounds".to_owned(),
            ));
        }

        if !(0..32).contains(&zoom) {
            return Err(LWGeomError::InvalidParameterError(
                "ST_TileEnvelope".to_owned(),
                "zoom".to_owned(),
            ));
        }

        let world_tile_size = 1 << zoom.min(31);
        if x < 0 || x >= world_tile_size {
            return Err(LWGeomError::InvalidParameterError(
                "ST_TileEnvelope".to_owned(),
                "x".to_owned(),
            ));
        }
        if y < 0 || y >= world_tile_size {
            return Err(LWGeomError::InvalidParameterError(
                "ST_TileEnvelope".to_owned(),
                "y".to_owned(),
            ));
        }

        let tile_geo_size_x = bounds_width / world_tile_size as f64;
        let tile_geo_size_y = bounds_height / world_tile_size as f64;

        let (x1, x2) = if (1.0 + margin * 2.0) > world_tile_size as f64 {
            (bbox.xmin(), bbox.xmax())
        } else {
            (
                bbox.xmin() + tile_geo_size_x * (x as f64 - margin),
                bbox.xmin() + tile_geo_size_x * (x as f64 + 1.0 + margin),
            )
        };
        let mut y1 = bbox.ymax() - tile_geo_size_y * (y as f64 + 1.0 + margin);
        let mut y2 = bbox.ymax() - tile_geo_size_y * (y as f64 - margin);
        if y1 < bbox.ymin() {
            y1 = bbox.ymin()
        }
        if y2 > bbox.ymax() {
            y2 = bbox.ymax()
        }

        Ok(LWPoly::construct_envelope(srid, x1, y1, x2, y2).into_lwgeom())
    }
}

impl LWGeomRef {
    pub fn has_srid(&self) -> bool {
        unsafe { lwgeom_has_srid(self.as_ptr()) != 0 }
    }

    pub fn get_srid(&self) -> Option<i32> {
        if self.has_srid() {
            Some(unsafe { lwgeom_get_srid(self.as_ptr()) })
        } else {
            None
        }
    }

    pub fn set_srid(&mut self, srid: i32) {
        unsafe { lwgeom_set_srid(self.as_mut_ptr(), srid) }
    }

    pub fn calculate_bbox(&self) -> GBox {
        let mut bbox = GBox::new_bbox();
        unsafe { lwgeom_calculate_gbox(self.as_ptr(), bbox.as_mut_ptr()) };
        bbox
    }
}

impl LWGeom {
    pub fn as_lwcollection(&self) -> &LWCollectionRef {
        let p_collection = unsafe { lwgeom_as_lwcollection(self.as_ptr()) };
        LWCollectionRef::from_ptr(p_collection)
    }

    pub fn get_type(&self) -> u32 {
        unsafe { lwgeom_get_type(self.as_ptr()) }
    }

    pub fn is_collection(&self) -> bool {
        unsafe { lwgeom_is_collection(self.as_ptr()) != LW_FALSE as c_int }
    }
}

impl LWGeomRef {
    pub fn as_lwcollection(&self) -> &LWCollectionRef {
        let p_collection = unsafe { lwgeom_as_lwcollection(self.as_ptr()) };
        LWCollectionRef::from_ptr(p_collection)
    }

    pub fn get_type(&self) -> u32 {
        unsafe { lwgeom_get_type(self.as_ptr()) }
    }

    pub fn is_collection(&self) -> bool {
        unsafe { lwgeom_is_collection(self.as_ptr()) != LW_FALSE as c_int }
    }
}

impl LWGeom {
    pub fn is_empty(&self) -> bool {
        unsafe { lwgeom_is_empty(self.as_ptr()) != LW_FALSE as c_int }
    }
}

impl LWGeomRef {
    pub fn is_empty(&self) -> bool {
        unsafe { lwgeom_is_empty(self.as_ptr()) != LW_FALSE as c_int }
    }
}
