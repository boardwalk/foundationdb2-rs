use crate::tuple::{TuplePack, TupleUnpack, UnpackError};

// const FLOAT_CODE: u8 = 0x20;
// const DOUBLE_CODE: u8 = 0x21;

impl TuplePack for f32 {
    fn pack(&self, out: &mut Vec<u8>, nested: bool) {
        unimplemented!()
    }
}

impl TupleUnpack for f32 {
    fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        unimplemented!()
    }
}

impl TuplePack for f64 {
    fn pack(&self, out: &mut Vec<u8>, nested: bool) {
        unimplemented!()
    }
}

impl TupleUnpack for f64 {
    fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        unimplemented!()
    }
}