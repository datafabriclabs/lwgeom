use lwgeom_sys::*;

pub struct GridSpec(*mut gridspec);

impl GridSpec {
    pub(crate) fn from_ptr(ptr: *mut gridspec) -> Self {
        debug_assert!(
            !ptr.is_null(),
            "Attempted to create a GridSpec from a null pointer."
        );
        Self(ptr)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut gridspec {
        self.0
    }

    pub(crate) fn as_ptr(&self) -> *const gridspec {
        self.0.cast_const()
    }

    pub(crate) fn as_ref(&self) -> &gridspec {
        unsafe { &*self.as_ptr() }
    }
}

impl GridSpec {
    pub fn new(
        (ipx, ipy, ipz, ipm): (f64, f64, f64, f64),
        (xsize, ysize, zsize, msize): (f64, f64, f64, f64),
    ) -> Self {
        let ffi_gridspec = Box::new(gridspec {
            ipx,
            ipy,
            ipz,
            ipm,
            xsize,
            ysize,
            zsize,
            msize,
        });
        let ptr = Box::into_raw(ffi_gridspec);
        Self::from_ptr(ptr)
    }
}

unsafe impl Send for GridSpec {}
unsafe impl Sync for GridSpec {}

impl Drop for GridSpec {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.as_mut_ptr())) }
    }
}
