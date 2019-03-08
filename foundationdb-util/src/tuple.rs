mod bool;
mod bytes;
mod float;
mod integer;
mod option;
mod tuple;
#[cfg(feature = "uuid")]
mod uuid;
mod versionstamp;

use std::convert::Infallible;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

pub use crate::tuple::tuple::Tuple;

pub trait Pack {
    fn pack(&self, out: &mut Vec<u8>, nested: bool);
}

pub trait Unpack: Sized {
    fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError>;
}

#[derive(Debug)]
pub enum UnpackError {
    WrongCode,
    OutOfData,
    OutOfRange,
    BadEncoding,
    // For Subspace::unpack
    MissingPrefix,
    TrailingData,
}

impl From<FromUtf8Error> for UnpackError {
    fn from(_: FromUtf8Error) -> Self {
        UnpackError::BadEncoding
    }
}

impl From<TryFromIntError> for UnpackError {
    fn from(_: TryFromIntError) -> Self {
        UnpackError::OutOfRange
    }
}

// Required by silly things like u64::try_from(u64)
impl From<Infallible> for UnpackError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

pub(crate) fn expect(inp: &[u8], expected: u8) -> Result<&[u8], UnpackError> {
    if let Some((&actual, inp)) = inp.split_first() {
        if actual == expected {
            Ok(inp)
        } else {
            Err(UnpackError::WrongCode)
        }
    } else {
        Err(UnpackError::OutOfData)
    }
}

#[cfg(test)]
mod tests {
    use crate::tuple::{Pack, Unpack};
    use std::fmt::Debug;

    fn test_pack_unpack<T>(in_val: T, buf: &mut Vec<u8>)
    where
        T: Pack + Unpack + Debug + PartialEq,
    {
        buf.clear();
        T::pack(&in_val, buf, false);
        let (out_val, rest) = T::unpack(&buf, false).unwrap();
        assert_eq!(in_val, out_val);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_bool() {
        let mut buf = Vec::new();
        test_pack_unpack(false, &mut buf);
        test_pack_unpack(true, &mut buf);
    }

    #[test]
    fn test_option() {
        let mut buf = Vec::new();
        test_pack_unpack(Some(32), &mut buf);
        let none_val: Option<i32> = None;
        test_pack_unpack(none_val, &mut buf);
    }

    #[test]
    fn test_tuple() {
        let mut buf = Vec::new();
        test_pack_unpack((42, true), &mut buf);
        test_pack_unpack((1, (2, 3)), &mut buf);
    }
}
