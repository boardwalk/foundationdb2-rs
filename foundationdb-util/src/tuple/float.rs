use crate::tuple::{Pack, Unpack, UnpackError};

// const FLOAT_CODE: u8 = 0x20;
// const DOUBLE_CODE: u8 = 0x21;

impl Pack for f32 {
    fn pack(&self, _out: &mut Vec<u8>, _nested: bool) {
        // TODO: Implement me
        unimplemented!()
    }
}

impl Unpack for f32 {
    fn unpack(_inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        // TODO: Implement me
        unimplemented!()
    }
}

impl Pack for f64 {
    fn pack(&self, _out: &mut Vec<u8>, _nested: bool) {
        // TODO: Implement me
        unimplemented!()
    }
}

impl Unpack for f64 {
    fn unpack(_inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        // TODO: Implement me
        unimplemented!()
    }
}

// TODO: Test f32 and f64
