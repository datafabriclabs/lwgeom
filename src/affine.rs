use lwgeom_sys::*;

pub struct Affine(*mut AFFINE);

impl Affine {
    pub(crate) fn from_ptr(ptr: *mut AFFINE) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a Affine from a null pointer."
        );
        Self(ptr)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut AFFINE {
        self.0
    }

    pub(crate) fn as_ptr(&self) -> *const AFFINE {
        self.0.cast_const()
    }

    pub(crate) fn as_ref(&self) -> &AFFINE {
        unsafe { &*self.as_ptr() }
    }
}

impl Affine {
    pub fn new(
        (afac, bfac, cfac, dfac, efac, ffac, gfac, hfac, ifac): (
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
        ),
        (xoff, yoff, zoff): (f64, f64, f64),
    ) -> Self {
        let ffi_affine = Box::new(AFFINE {
            afac,
            bfac,
            cfac,
            dfac,
            efac,
            ffac,
            gfac,
            hfac,
            ifac,
            xoff,
            yoff,
            zoff,
        });
        let ptr = Box::into_raw(ffi_affine);
        Self::from_ptr(ptr)
    }
}

unsafe impl Send for Affine {}
unsafe impl Sync for Affine {}

impl Drop for Affine {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.as_mut_ptr())) }
    }
}
