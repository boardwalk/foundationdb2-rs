mod bool;
mod bytes;
mod float;
mod integer;
mod option;
mod tuple;
#[cfg(feature = "uuid")]
mod uuid;
mod versionstamp;

use std::string::FromUtf8Error;
use std::num::TryFromIntError;

#[derive(Debug)]
pub enum UnpackError {
    WrongCode,
    OutOfData,
    OutOfRange,
    BadEncoding,
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
impl From<()> for UnpackError {
    fn from(_: ()) -> Self {
        unreachable!()
    }
}

// nested = are we inside a tuple?
// false at the top level

pub trait TuplePack {
    fn pack(&self, out: &mut Vec<u8>, nested: bool);
}

pub trait TupleUnpack: Sized {
    fn unpack(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError>;
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
    use rand::{random, thread_rng, Rng};
    use rand::distributions::{Alphanumeric, Standard};
    use std::fmt::Debug;
    use super::{TuplePack, TupleUnpack};

    fn test_pack_unpack<Tin, Tout>(in_val: Tin, buf: &mut Vec<u8>)
    where
        Tin: TuplePack + Debug,
        Tout: TupleUnpack + Debug,
        Tin: PartialEq<Tout>,
    {
        buf.clear();
        TuplePack::pack(&in_val, buf, false);
        let (out_val, rest) = <Tout as TupleUnpack>::unpack(&buf, false).unwrap();
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
    fn test_bytes() {
        let mut buf = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..10000 {
            let nbytes = rng.gen_range(0, 64);
            let in_val = rng.sample_iter(&Standard).take(nbytes).collect::<Vec<u8>>();

            buf.clear();
            TuplePack::pack(&in_val[..], &mut buf, false);

            let (out_val, rest) = <Vec<u8> as TupleUnpack>::unpack(&buf, false).unwrap();

            assert_eq!(in_val, out_val);
            assert!(rest.is_empty());
        }
    }

    // TODO: Test floats

    #[test]
    fn test_string() {
        let mut buf = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..10000 {
            let nchars = rng.gen_range(0, 64);
            let in_val = rng.sample_iter(&Alphanumeric).take(nchars).collect::<String>();

            buf.clear();
            TuplePack::pack(in_val.as_str(), &mut buf, false);

            let (out_val, rest) = <String as TupleUnpack>::unpack(&buf, false).unwrap();

            assert_eq!(in_val, out_val);
            assert!(rest.is_empty());
        }
    }
}