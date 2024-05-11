use core::cell::UnsafeCell;
use core::ffi::CStr;
use core::fmt;
use core::marker::PhantomData;

use lwgeom_sys::*;

pub struct GBox(*mut GBOX);

impl GBox {
    pub fn from_ptr(ptr: *mut GBOX) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a GBox from a null pointer."
        );
        GBox(ptr)
    }

    fn as_ptr(&self) -> *mut GBOX {
        self.0
    }

    fn as_ref(&self) -> &GBOX {
        unsafe { &*self.as_ptr().cast_const() }
    }
}

impl fmt::Display for GBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c_gbox_string = unsafe { CStr::from_ptr(gbox_to_string(self.as_ptr())) };
        c_gbox_string.to_string_lossy().fmt(f)
    }
}

unsafe impl Send for GBox {}
unsafe impl Sync for GBox {}

impl Drop for GBox {
    fn drop(&mut self) {
        unsafe {
            lwfree(self.as_ptr().cast());
        }
    }
}

pub struct GBoxRef(PhantomData<UnsafeCell<*mut GBOX>>);

impl GBoxRef {
    pub fn from_ptr<'a>(ptr: *mut GBOX) -> &'a Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a GBoxRef from a null pointer."
        );
        unsafe { &*(ptr as *mut _) }
    }

    fn as_ptr(&self) -> *mut GBOX {
        self as *const _ as *mut _
    }

    fn as_ref(&self) -> &GBOX {
        unsafe { &*self.as_ptr().cast_const() }
    }
}

impl fmt::Display for GBoxRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c_gbox_string = unsafe { CStr::from_ptr(gbox_to_string(self.as_ptr())) };
        c_gbox_string.to_string_lossy().fmt(f)
    }
}

unsafe impl Send for GBoxRef {}
unsafe impl Sync for GBoxRef {}

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

impl GBoxRef {
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
