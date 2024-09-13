use std::cell::UnsafeCell;
use std::marker::PhantomData;

use lwgeom_sys::*;

pub struct LWCollection(*mut LWCOLLECTION);

impl LWCollection {
    pub fn from_ptr(ptr: *mut LWCOLLECTION) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWCollection from a null pointer."
        );
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut LWCOLLECTION {
        self.0
    }
}

unsafe impl Send for LWCollection {}
unsafe impl Sync for LWCollection {}

impl Drop for LWCollection {
    fn drop(&mut self) {
        unsafe { lwcollection_free(self.as_ptr()) }
    }
}

pub struct LWCollectionRef(PhantomData<UnsafeCell<*mut LWCOLLECTION>>);

impl LWCollectionRef {
    pub fn from_ptr<'a>(ptr: *mut LWCOLLECTION) -> &'a Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a LWCollectionRef from a null pointer."
        );
        unsafe { &*(ptr as *mut _) }
    }

    fn as_ptr(&self) -> *mut LWCOLLECTION {
        self as *const _ as *mut _
    }

    pub fn as_ref(&self) -> &LWCOLLECTION {
        unsafe { &*self.as_ptr().cast_const() }
    }
}

unsafe impl Send for LWCollectionRef {}
unsafe impl Sync for LWCollectionRef {}
