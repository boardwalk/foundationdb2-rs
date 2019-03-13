use crate::tuple::{expect, Pack, Unpack, UnpackError};
use std::convert::TryFrom;
use std::mem::size_of;
use uuid::{Bytes, Uuid};

const UUID_CODE: u8 = 0x30;

impl Pack for Uuid {
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(UUID_CODE);
        out.extend_from_slice(self.as_bytes());
    }
}

impl Unpack for Uuid {
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

#[cfg(test)]
mod test {
    use crate::tuple::test::test_pack_unpack;
    use uuid::Uuid;

    #[test]
    fn test_pack_unpack_uuid() {
        let mut buf = Vec::new();
        let uuid = Uuid::parse_str("0436430c-2b02-624c-2032-570501212b57").unwrap();
        test_pack_unpack(uuid, &mut buf);
    }
}
