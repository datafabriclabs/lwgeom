use lwgeom_sys::*;

use crate::foreign_type::ForeignType;
use crate::{GBox, LWGeom};

impl LWGeom {
    pub fn as_mvt_geom(
        &mut self, gbox: &GBox, extend: u32, buffer: u32, clip_geom: bool,
    ) -> Option<Self> {
        let ptr = unsafe { mvt_geom(self.as_mut_ptr(), gbox.as_ptr(), extend, buffer, clip_geom) };
        if ptr.is_null() {
            None
        } else {
            Some(Self::from_ptr(ptr))
        }
    }
}
