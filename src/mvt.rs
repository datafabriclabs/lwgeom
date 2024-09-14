use lwgeom_sys::*;

use crate::affine::Affine;
use crate::grid_spec::GridSpec;
use crate::{GBox, LWGeom, LWGeomError, LWGeomRef, Result};

impl LWGeom {
    fn get_basic_type(&self) -> Result<u32> {
        match self.as_ref().type_ as u32 {
            type_ @ (POINTTYPE | LINETYPE | POLYGONTYPE) => Ok(type_),
            TRIANGLETYPE => Ok(POLYGONTYPE),
            type_ @ (MULTIPOINTTYPE | MULTILINETYPE | MULTIPOLYGONTYPE) => Ok(type_ - 3),
            COLLECTIONTYPE | TINTYPE => {
                let mut type_ = 0;
                let ffi_collection = self.as_lwcollection().as_ref();
                for i in 0..ffi_collection.ngeoms {
                    let p_sg = unsafe { *ffi_collection.geoms.add(i as usize) };
                    let sg_ref = LWGeomRef::from_ptr(p_sg);
                    type_ = type_.max(sg_ref.get_basic_type()?);
                }
                Ok(type_)
            }
            type_ => Err(LWGeomError::InvalidGeomtryBasicType(type_)),
        }
    }

    fn to_basic_type(&mut self, original_type: u8) -> LWGeomRef {
        let geom_out = LWGeomRef::from_mut_ptr(self.as_mut_ptr());
        if COLLECTIONTYPE == self.get_type() {
            let g = self.as_lwcollection();
            let geom_out = g.extract(original_type as u32).into_lwgeom();
        }
        todo!()
    }
}

impl LWGeomRef {
    fn get_basic_type(&self) -> Result<u32> {
        match self.as_ref().type_ as u32 {
            type_ @ (POINTTYPE | LINETYPE | POLYGONTYPE) => Ok(type_),
            TRIANGLETYPE => Ok(POLYGONTYPE),
            type_ @ (MULTIPOINTTYPE | MULTILINETYPE | MULTIPOLYGONTYPE) => Ok(type_ - 3),
            COLLECTIONTYPE | TINTYPE => {
                let mut type_ = 0;
                let ffi_collection = self.as_lwcollection().as_ref();
                for i in 0..ffi_collection.ngeoms {
                    let p_sg = unsafe { *ffi_collection.geoms.add(i as usize) };
                    let sg_ref = LWGeomRef::from_ptr(p_sg);
                    type_ = type_.max(sg_ref.get_basic_type()?);
                }
                Ok(type_)
            }
            type_ => Err(LWGeomError::InvalidGeomtryBasicType(type_)),
        }
    }
}

impl LWGeom {
    pub fn mvt_geom(
        &mut self, bounds: &GBox, extent: u32, buffer: u32, clip_geom: bool,
    ) -> Result<Self> {
        let affine = Affine::new(
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
        );
        let grid = GridSpec::new((0.0, 0.0, 0.0, 0.0), (1.0, 1.0, 0.0, 0.0));
        let width = bounds.xmax() - bounds.xmin();
        let height = bounds.ymax() - bounds.ymin();
        let fx: f64;
        let fy: f64;
        let basic_type = self.get_basic_type()?;
        let preserve_collapsed = LW_FALSE;

        todo!()
    }
}
