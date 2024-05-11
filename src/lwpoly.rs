use lwgeom_sys::*;

use crate::LWGeom;

pub struct LWPoly(*mut LWPOLY);

impl LWPoly {
    pub fn from_ptr(ptr: *mut LWPOLY) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWPoly from a null pointer."
        );
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut LWPOLY {
        self.0
    }

    fn into_ptr(self) -> *mut LWPOLY {
        let ptr = self.0;
        core::mem::forget(self);
        ptr
    }
}

unsafe impl Send for LWPoly {}
unsafe impl Sync for LWPoly {}

impl Drop for LWPoly {
    fn drop(&mut self) {
        unsafe { lwpoly_free(self.as_ptr()) };
    }
}

impl LWPoly {
    pub fn into_lwgeom(self) -> LWGeom {
        let p_geom = unsafe { lwpoly_as_lwgeom(self.into_ptr()) };
        LWGeom::from_ptr(p_geom)
    }

    pub fn construct_envelope(srid: i32, x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let p_poly = unsafe { lwpoly_construct_envelope(srid, x1, y1, x2, y2) };
        Self::from_ptr(p_poly)
    }
}
