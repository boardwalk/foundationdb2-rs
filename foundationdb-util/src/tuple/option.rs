use crate::tuple::{Pack, Unpack, UnpackError};

const NULL_CODE: u8 = 0x00;

impl<T> Pack for Option<T>
where
    T: Pack,
{
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        if let Some(v) = self {
            T::pack(v, out, false)
        } else {
            out.push(NULL_CODE);
        }
    }
}

impl<T> Unpack for Option<T>
where
    T: Unpack,
{
    fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        if let Some((&code, inp_some)) = inp.split_first() {
            if code == NULL_CODE {
                Ok((None, inp_some))
            } else {
                match T::unpack(inp, nested) {
                    Ok((v, inp)) => Ok((Some(v), inp)),
                    Err(err) => Err(err),
                }
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
    fn test_pack_unpack_option() {
        let mut buf = Vec::new();
        test_pack_unpack(Some(32), &mut buf);
        let none_val: Option<i32> = None;
        test_pack_unpack(none_val, &mut buf);
    }
}
