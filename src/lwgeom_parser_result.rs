use core::ffi::CStr;

use lwgeom_sys::*;

use crate::LWGeom;
use crate::foreign_type::ForeignType;

pub struct LWGeomParserResult(*mut LWGEOM_PARSER_RESULT);

impl ForeignType for LWGeomParserResult {
    type FFIType = LWGEOM_PARSER_RESULT;

    fn from_ptr(ptr: *mut Self::FFIType) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWGeomParserResult from a null pointer."
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

unsafe impl Send for LWGeomParserResult {}
unsafe impl Sync for LWGeomParserResult {}

impl Drop for LWGeomParserResult {
    fn drop(&mut self) {
        unsafe { lwgeom_parser_result_free(self.as_mut_ptr()) }
    }
}

impl LWGeomParserResult {
    pub fn take_geom(&mut self) -> LWGeom {
        let p_geom = self.as_mut_ref().geom;
        self.as_mut_ref().geom = core::ptr::null_mut();
        LWGeom::from_ptr(p_geom)
    }

    pub fn message(&self) -> Option<String> {
        let c_message = self.as_ref().message;
        if c_message.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(c_message) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}
