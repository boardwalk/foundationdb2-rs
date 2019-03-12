use crate::tuple::{expect, Pack, Unpack, UnpackError};

const NESTED_CODE: u8 = 0x05;

pub trait Tuple {}

macro_rules! impl_tuple {
    ( $($id:ident : $ty:tt),* ) => {
        impl< $($ty),* > Pack for ( $($ty,)* )
        where
            $($ty: Pack,)*
        {
            fn pack(&self, out: &mut Vec<u8>, nested: bool) {
                let ( $($id,)* ) = self;
                if nested {
                    out.push(NESTED_CODE);
                }
                $($ty::pack($id, out, true);)*
                if nested {
                    out.push(0x00);
                }
            }
        }

        impl< $($ty,)* > Unpack for ( $($ty,)* )
        where
            $($ty: Unpack,)*
        {
            fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError> {
                let inp = if nested {
                    expect(inp, NESTED_CODE)?
                } else {
                    inp
                };
                $(let ($id, inp) = $ty::unpack(inp, true)?;)*
                let inp = if nested {
                    expect(inp, 0x00)?
                } else {
                    inp
                };
                Ok((( $($id,)* ), inp))
            }
        }

        impl< $($ty,)* > Tuple for ( $($ty,)* )
        {}
    };
}

impl_tuple!();
impl_tuple!(v1: T1);
impl_tuple!(v1: T1, v2: T2);
impl_tuple!(v1: T1, v2: T2, v3: T3);
impl_tuple!(v1: T1, v2: T2, v3: T3, v4: T4);
impl_tuple!(v1: T1, v2: T2, v3: T3, v4: T4, v5: T5);
impl_tuple!(v1: T1, v2: T2, v3: T3, v4: T4, v5: T5, v6: T6);
impl_tuple!(v1: T1, v2: T2, v3: T3, v4: T4, v5: T5, v6: T6, v7: T7);
impl_tuple!(v1: T1, v2: T2, v3: T3, v4: T4, v5: T5, v6: T6, v7: T7, v8: T8);

#[cfg(test)]
mod test {
    use crate::tuple::test::test_pack_unpack;

    #[test]
    fn test_pack_unpack_tuple() {
        let mut buf = Vec::new();
        test_pack_unpack((42, true), &mut buf);
        test_pack_unpack((1, (2, 3)), &mut buf);
    }
}
