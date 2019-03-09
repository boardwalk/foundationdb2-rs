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

// Allows you to pack a &&str, f.e., which happens in the case of tuples like (&str,)
impl<T> Pack for &T
where
    T: Pack + ?Sized,
{
    fn pack(&self, out: &mut Vec<u8>, nested: bool) {
        T::pack(*self, out, nested)
    }
}

#[derive(Debug, PartialEq)]
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
mod test {
    use crate::tuple::{Pack, Unpack};
    use std::fmt::Debug;

    pub fn test_pack<T>(in_val: T, buf: &mut Vec<u8>, out_val: &[u8])
    where
        T: Pack,
    {
        buf.clear();
        T::pack(&in_val, buf, false);
        assert_eq!(&buf[..], out_val);
    }

    pub fn test_pack_unpack<T>(in_val: T, buf: &mut Vec<u8>)
    where
        T: Pack + Unpack + Debug + PartialEq,
    {
        buf.clear();
        T::pack(&in_val, buf, false);
        let (out_val, rest) = T::unpack(&buf, false).unwrap();
        assert_eq!(in_val, out_val);
        assert!(rest.is_empty());
    }
}
