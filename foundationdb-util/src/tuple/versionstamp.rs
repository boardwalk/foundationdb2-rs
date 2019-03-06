use crate::tuple::{Pack, Unpack, UnpackError};

// const VERSIONSTAMP_CODE: u8 = 0x33;

pub struct Versionstamp;

impl Pack for Versionstamp {
    fn pack(&self, _out: &mut Vec<u8>, _nested: bool) {
        // TODO: Implement me
        unimplemented!()
    }
}

impl Unpack for Versionstamp {
    fn unpack(_inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        // TODO: Implement me
        unimplemented!()
    }
}

// TODO: Test Versionstamp
