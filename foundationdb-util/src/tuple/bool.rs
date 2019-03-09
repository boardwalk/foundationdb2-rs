use crate::tuple::{Pack, Unpack, UnpackError};

const FALSE_CODE: u8 = 0x26;
const TRUE_CODE: u8 = 0x27;

impl Pack for bool {
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(if *self { TRUE_CODE } else { FALSE_CODE });
    }
}

impl Unpack for bool {
    fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        if let Some((&code, inp)) = inp.split_first() {
            if code == TRUE_CODE {
                Ok((true, inp))
            } else if code == FALSE_CODE {
                Ok((false, inp))
            } else {
                Err(UnpackError::WrongCode)
            }
        } else {
            Err(UnpackError::OutOfData)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tuple::test::test_pack_unpack;

    #[test]
    fn test_pack_unpack_bool() {
        let mut buf = Vec::new();
        test_pack_unpack(false, &mut buf);
        test_pack_unpack(true, &mut buf);
    }
}