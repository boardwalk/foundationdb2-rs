use crate::tuple::{expect, Pack, Unpack, UnpackError};

const NESTED_CODE: u8 = 0x05;

impl<T1, T2> Pack for (T1, T2)
where
    T1: Pack,
    T2: Pack,
{
    fn pack(&self, out: &mut Vec<u8>, nested: bool) {
        let (v1, v2) = self;
        if nested {
            out.push(NESTED_CODE);
        }
        T1::pack(v1, out, true);
        T2::pack(v2, out, true);
        if nested {
            out.push(0x00);
        }
    }
}

impl<T1, T2> Unpack for (T1, T2)
where
    T1: Unpack,
    T2: Unpack,
{
    fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = if nested {
            expect(inp, NESTED_CODE)?
        } else {
            inp
        };
        let (v1, inp) = T1::unpack(inp, true)?;
        let (v2, inp) = T2::unpack(inp, true)?;
        let inp = if nested {
            expect(inp, 0x00)?
        } else {
            inp
        };
        Ok(((v1, v2), inp))
    }
}

// TODO: Implement higher arity
