use crate::tuple::{expect, TuplePack, TupleUnpack, UnpackError};

const NESTED_CODE: u8 = 0x05;

impl<T1, T2> TuplePack for (T1, T2)
where
    T1: TuplePack,
    T2: TuplePack,
{
    fn pack(&self, out: &mut Vec<u8>, nested: bool) {
        let (v1, v2) = self;
        if nested { out.push(NESTED_CODE);  }
        T1::pack(v1, out, true);
        T2::pack(v2, out, true);
        if nested { out.push(0x00); }
    }
}

impl<T1, T2> TupleUnpack for (T1, T2)
where
    T1: TupleUnpack,
    T2: TupleUnpack,
{
    fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = if nested { expect(inp, NESTED_CODE)? } else { inp };
        let (v1, inp) = T1::unpack(inp, true)?;
        let (v2, inp) = T2::unpack(inp, true)?;
        let inp = if nested { expect(inp, 0x00)? } else { inp };
        Ok(((v1, v2), inp))
    }
}

// TODO: Implement higher arity