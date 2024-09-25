use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem;

pub(crate) trait ForeignType: Sized + Send + Sync {
    type FFIType;

    fn from_ptr(ptr: *mut Self::FFIType) -> Self;

    fn as_mut_ptr(&mut self) -> *mut Self::FFIType;

    fn as_ptr(&self) -> *const Self::FFIType;

    fn as_ref(&self) -> &Self::FFIType {
        unsafe { &*self.as_ptr() }
    }

    fn as_mut_ref(&mut self) -> &mut Self::FFIType {
        unsafe { &mut *self.as_mut_ptr() }
    }

    fn into_ptr(mut self) -> *mut Self::FFIType {
        let ptr = self.as_mut_ptr();
        mem::forget(self);
        ptr
    }
}

pub(crate) trait ForeignTypeRef: Sized + Send + Sync {
    type FFIType;

    fn from_ptr<'a>(ptr: *const Self::FFIType) -> &'a Self {
        debug_assert!(!ptr.is_null());
        unsafe { &*(ptr as *const _) }
    }

    fn as_mut_ptr(&mut self) -> *mut Self::FFIType {
        self as *const _ as _
    }

    fn as_ptr(&self) -> *const Self::FFIType {
        self as *const _ as _
    }
}

pub(crate) type Opaque = PhantomData<UnsafeCell<*mut ()>>;
