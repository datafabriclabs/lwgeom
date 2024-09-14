use core::ffi::CStr;
use core::fmt;

use lwgeom_sys::*;

pub struct GBox(*mut GBOX);

impl GBox {
    pub(crate) fn from_ptr(ptr: *mut GBOX) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a GBox from a null pointer."
        );
        Self(ptr)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut GBOX {
        self.0
    }

    pub(crate) fn as_ptr(&self) -> *const GBOX {
        self.0.cast_const()
    }

    pub(crate) fn as_ref(&self) -> &GBOX {
        unsafe { &*self.as_ptr() }
    }
}

impl GBox {
    pub fn new_bbox() -> Self {
        let ffi_gbox = Box::new(GBOX {
            flags: LWFLAG_BBOX as u16,
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
            zmin: 0.0,
            zmax: 0.0,
            mmin: 0.0,
            mmax: 0.0,
        });
        let ptr = Box::into_raw(ffi_gbox);
        Self::from_ptr(ptr)
    }
}

impl fmt::Display for GBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p_gbox_string = unsafe { gbox_to_string(self.as_ptr()) };
        let c_gbox_string = unsafe { CStr::from_ptr(p_gbox_string) };
        c_gbox_string.to_string_lossy().fmt(f)?;
        unsafe {
            lwfree(p_gbox_string.cast());
        }
        Ok(())
    }
}

unsafe impl Send for GBox {}
unsafe impl Sync for GBox {}

impl Drop for GBox {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.as_mut_ptr())) }
    }
}

impl GBox {
    pub fn xmin(&self) -> f64 {
        self.as_ref().xmin
    }

    pub fn xmax(&self) -> f64 {
        self.as_ref().xmax
    }

    pub fn ymin(&self) -> f64 {
        self.as_ref().ymin
    }

    pub fn ymax(&self) -> f64 {
        self.as_ref().ymax
    }
}
