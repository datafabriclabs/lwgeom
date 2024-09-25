use lwgeom_sys::*;

use crate::foreign_type::{ForeignType, ForeignTypeRef, Opaque};
use crate::{LWGeom, LWGeomRef};

pub struct LWCollection(*mut LWCOLLECTION);

impl ForeignType for LWCollection {
    type FFIType = LWCOLLECTION;

    fn from_ptr(ptr: *mut Self::FFIType) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWCollection from a null pointer."
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

unsafe impl Send for LWCollection {}
unsafe impl Sync for LWCollection {}

impl Drop for LWCollection {
    fn drop(&mut self) {
        unsafe { lwcollection_free(self.as_mut_ptr()) }
    }
}

pub struct LWCollectionRef(Opaque);

impl ForeignTypeRef for LWCollectionRef {
    type FFIType = LWCOLLECTION;
}

unsafe impl Send for LWCollectionRef {}
unsafe impl Sync for LWCollectionRef {}

impl LWCollection {
    pub fn into_lwgeom(self) -> LWGeom {
        let p_geom = unsafe { lwcollection_as_lwgeom(self.into_ptr()) };
        LWGeom::from_ptr(p_geom)
    }

    pub fn as_lwgeom(&self) -> &LWGeomRef {
        let p_geom = unsafe { lwcollection_as_lwgeom(self.as_ptr()) };
        LWGeomRef::from_ptr(p_geom)
    }
}

impl LWCollection {
    pub fn extract(&self, type_: u32) -> Self {
        let p_collection = unsafe { lwcollection_extract(self.as_ptr(), type_) };
        Self::from_ptr(p_collection)
    }
}

impl LWCollectionRef {
    pub fn extract(&self, type_: u32) -> LWCollection {
        let p_collection = unsafe { lwcollection_extract(self.as_ptr(), type_) };
        LWCollection::from_ptr(p_collection)
    }
}
