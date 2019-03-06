use crate::tuple::{expect, TuplePack, TupleUnpack, UnpackError};
use std::convert::TryFrom;
use std::mem::size_of;
use uuid::{Bytes, Uuid};

const UUID_CODE: u8 = 0x30;

impl TuplePack for Uuid {
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(UUID_CODE);
        out.extend_from_slice(self.as_bytes());
    }
}

impl TupleUnpack for Uuid {
    fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = expect(inp, UUID_CODE)?;

        if inp.len() < size_of::<Bytes>() {
            return Err(UnpackError::OutOfData);
        }

        let (out, inp) = inp.split_at(size_of::<Bytes>());
        let out = Uuid::from_bytes(Bytes::try_from(out).unwrap());
        Ok((out, inp))
    }
}

// TODO: Test Uuid