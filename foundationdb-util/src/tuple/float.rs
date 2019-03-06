use crate::tuple::{TuplePack, TupleUnpack, UnpackError};

// const FLOAT_CODE: u8 = 0x20;
// const DOUBLE_CODE: u8 = 0x21;

impl TuplePack for f32 {
    fn pack(&self, _out: &mut Vec<u8>, _nested: bool) {
        // TODO: Implement me
        unimplemented!()
    }
}

impl TupleUnpack for f32 {
    fn unpack(_inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        // TODO: Implement me
        unimplemented!()
    }
}

impl TuplePack for f64 {
    fn pack(&self, _out: &mut Vec<u8>, _nested: bool) {
        // TODO: Implement me
        unimplemented!()
    }
}

impl TupleUnpack for f64 {
    fn unpack(_inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        // TODO: Implement me
        unimplemented!()
    }
}

// TODO: Test f32 and f64