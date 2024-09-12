use lwgeom_sys::*;

pub struct Affine(*mut AFFINE);

impl Affine {
    pub fn from_ptr(ptr: *mut AFFINE) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a Affine from a null pointer."
        );
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut AFFINE {
        self.0
    }

    fn as_ref(&self) -> &AFFINE {
        unsafe { &*self.as_ptr().cast_const() }
    }
}
