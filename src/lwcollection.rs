use std::cell::UnsafeCell;
use std::marker::PhantomData;

use lwgeom_sys::*;

use crate::LWGeom;

pub struct LWCollection(*mut LWCOLLECTION);

impl LWCollection {
    pub(crate) fn from_ptr(ptr: *mut LWCOLLECTION) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWCollection from a null pointer."
        );
        Self(ptr)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut LWCOLLECTION {
        self.0
    }

    pub(crate) fn as_ptr(&self) -> *const LWCOLLECTION {
        self.0.cast_const()
    }

    pub(crate) fn into_ptr(self) -> *mut LWCOLLECTION {
        let ptr = self.0;
        core::mem::forget(self);
        ptr
    }

    pub(crate) fn as_ref(&self) -> &LWCOLLECTION {
        unsafe { &*self.as_ptr() }
    }
}

unsafe impl Send for LWCollection {}
unsafe impl Sync for LWCollection {}

impl Drop for LWCollection {
    fn drop(&mut self) {
        unsafe { lwcollection_free(self.as_mut_ptr()) }
    }
}

pub struct LWCollectionRef(PhantomData<UnsafeCell<()>>);

impl LWCollectionRef {
    pub(crate) fn from_ptr<'a>(ptr: *const LWCOLLECTION) -> &'a Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWCollectionRef from a null pointer."
        );
        unsafe { &*(ptr as *const _) }
    }

    pub(crate) fn from_mut_ptr<'a>(ptr: *mut LWCOLLECTION) -> &'a mut Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a mutable LWCollectionRef from a null pointer."
        );
        unsafe { &mut *(ptr as *mut _) }
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut LWCOLLECTION {
        self as *const _ as _
    }

    pub(crate) fn as_ptr(&self) -> *const LWCOLLECTION {
        self as *const _ as _
    }

    pub(crate) fn as_ref(&self) -> &LWCOLLECTION {
        unsafe { &*self.as_ptr() }
    }
}

unsafe impl Send for LWCollectionRef {}
unsafe impl Sync for LWCollectionRef {}

impl LWCollection {
    pub fn into_lwgeom(self) -> LWGeom {
        let p_geom = unsafe { lwcollection_as_lwgeom(self.into_ptr()) };
        LWGeom::from_ptr(p_geom)
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
