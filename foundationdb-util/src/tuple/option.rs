use crate::tuple::{TuplePack, TupleUnpack, UnpackError};

const NULL_CODE: u8 = 0x00;

impl<T> TuplePack for Option<T>
where
    T: TuplePack,
{
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        if let Some(v) = self {
            T::pack(v, out, false)
        } else {
            out.push(NULL_CODE);
        }
    }
}

impl<T> TupleUnpack for Option<T>
where
    T: TupleUnpack,
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
