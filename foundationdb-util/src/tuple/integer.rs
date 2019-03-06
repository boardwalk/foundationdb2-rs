use crate::tuple::{Pack, Unpack, UnpackError};
use byteorder::{BigEndian, ByteOrder};
use std::convert::TryFrom;

// This value is used (unused by us) for a 9+ byte negative integer
const NEG_INT_START: u8 = 0x0b;
// This value is used for exactly zero
const INT_ZERO_CODE: u8 = 0x14;
// This value is used (unused by us) for a 9+ byte positive integer
const POS_INT_END: u8 = 0x1d;

fn pack_int(inp: i64, out: &mut Vec<u8>) {
    let nbytes = ((64 - inp.wrapping_abs().leading_zeros() + 7) / 8) as usize;

    let code = if inp >= 0 {
        INT_ZERO_CODE + nbytes as u8
    } else {
        INT_ZERO_CODE - nbytes as u8
    };

    let val = if inp >= 0 {
        inp
    } else {
        let max_value = 1i64
            .checked_shl(nbytes as u32 * 8)
            .unwrap_or(0)
            .wrapping_sub(1);
        max_value.wrapping_add(inp)
    };

    let mut buf = [0; 8];
    BigEndian::write_i64(&mut buf, val);

    out.push(code);
    out.extend_from_slice(&buf[8 - nbytes..]);
}

fn pack_uint(inp: u64, out: &mut Vec<u8>) {
    let nbytes = ((64 - inp.leading_zeros() + 7) / 8) as usize;

    let code = INT_ZERO_CODE + nbytes as u8;

    let mut buf = [0; 8];
    BigEndian::write_u64(&mut buf, inp);

    out.push(code);
    out.extend_from_slice(&buf[8 - nbytes..]);
}

fn unpack_int(inp: &[u8]) -> Result<(i64, &[u8]), UnpackError> {
    if let Some((&code, inp)) = inp.split_first() {
        if code > NEG_INT_START && code < POS_INT_END {
            let nbytes = if code >= INT_ZERO_CODE {
                (code - INT_ZERO_CODE) as usize
            } else {
                (INT_ZERO_CODE - code) as usize
            };

            if inp.len() < nbytes {
                return Err(UnpackError::OutOfData);
            }

            let mut buf = [0; 8];
            buf[8 - nbytes..].copy_from_slice(&inp[..nbytes]);
            let val = BigEndian::read_i64(&buf);

            let out = if code >= INT_ZERO_CODE {
                val
            } else {
                let max_value = 1i64
                    .checked_shl(nbytes as u32 * 8)
                    .unwrap_or(0)
                    .wrapping_sub(1);
                val.wrapping_sub(max_value)
            };

            Ok((out, &inp[nbytes..]))
        } else {
            Err(UnpackError::WrongCode)
        }
    } else {
        Err(UnpackError::OutOfData)
    }
}

fn unpack_uint(inp: &[u8]) -> Result<(u64, &[u8]), UnpackError> {
    if let Some((&code, inp)) = inp.split_first() {
        if code >= INT_ZERO_CODE && code < POS_INT_END {
            let nbytes = (code - INT_ZERO_CODE) as usize;

            if inp.len() < nbytes {
                return Err(UnpackError::OutOfData);
            }

            let mut buf = [0; 8];
            buf[8 - nbytes..].copy_from_slice(&inp[..nbytes]);
            let out = BigEndian::read_u64(&buf);

            Ok((out, &inp[nbytes..]))
        } else {
            Err(UnpackError::WrongCode)
        }
    } else {
        Err(UnpackError::OutOfData)
    }
}

macro_rules! impl_i {
    ($ty:ty) => {
        impl Pack for $ty {
            fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
                pack_int(i64::from(*self), out)
            }
        }

        impl Unpack for $ty {
            fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
                let (out, inp) = unpack_int(inp)?;
                let out = TryFrom::try_from(out)?;
                Ok((out, inp))
            }
        }
    };
}

macro_rules! impl_u {
    ($ty:ty) => {
        impl Pack for $ty {
            fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
                pack_uint(u64::from(*self), out)
            }
        }

        impl Unpack for $ty {
            fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
                let (out, inp) = unpack_uint(inp)?;
                let out = TryFrom::try_from(out)?;
                Ok((out, inp))
            }
        }
    };
}

impl_i!(i8);
impl_i!(i16);
impl_i!(i32);
impl_i!(i64);

impl_u!(u8);
impl_u!(u16);
impl_u!(u32);
impl_u!(u64);

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    fn test_int(in_val: i64, buf: &mut Vec<u8>) {
        buf.clear();
        pack_int(in_val, buf);
        let (out_val, rest) = unpack_int(buf).unwrap();
        assert_eq!(in_val, out_val);
        assert!(rest.is_empty());
    }

    fn test_uint(in_val: u64, buf: &mut Vec<u8>) {
        buf.clear();
        pack_uint(in_val, buf);
        let (out_val, rest) = unpack_uint(buf).unwrap();
        assert_eq!(in_val, out_val);
        assert!(rest.is_empty());
    }

    fn test_pack_int(in_val: i64, out_bytes: &[u8]) {
        let mut buf = Vec::new();
        pack_int(in_val, &mut buf);
        assert_eq!(&buf[..], out_bytes);
    }

    fn test_pack_uint(in_val: u64, out_bytes: &[u8]) {
        let mut buf = Vec::new();
        pack_uint(in_val, &mut buf);
        assert_eq!(&buf[..], out_bytes);
    }

    #[test]
    fn pack_unpack_int() {
        let mut buf = Vec::new();
        test_int(i64::min_value(), &mut buf);
        test_int(i64::min_value() + 1, &mut buf);
        test_int(i64::max_value(), &mut buf);
        test_int(i64::max_value() - 1, &mut buf);
        test_int(0, &mut buf);
        test_int(1, &mut buf);
        test_int(-1, &mut buf);
        for _ in 0..100000 {
            let in_val = random::<i64>();
            test_int(in_val, &mut buf);
        }
    }

    #[test]
    fn pack_unpack_uint() {
        let mut buf = Vec::new();
        test_uint(u64::min_value(), &mut buf);
        test_uint(u64::min_value() + 1, &mut buf);
        test_uint(u64::max_value(), &mut buf);
        test_uint(u64::max_value() - 1, &mut buf);
        for _ in 0..100000 {
            let in_val = random::<u64>();
            test_uint(in_val, &mut buf);
        }
    }

    #[test]
    fn pack_int_testcases() {
        // PYTHONPATH=$FDB_PATH/bindings/python python3
        // from fdb.tuple import _encode
        // from binascii import hexlify
        // (b, _) = _encode(x); print(hexlify(b))
        test_pack_int(-314159265, &[0x10, 0xed, 0x46, 0x4f, 0x5e]);
        test_pack_int(-2, &[0x13, 0xfd]);
        test_pack_int(-1, &[0x13, 0xfe]);
        test_pack_int(0, &[0x14]);
        test_pack_int(1, &[0x15, 0x01]);
        test_pack_int(2, &[0x15, 0x02]);
        test_pack_int(123, &[0x15, 0x7b]);
        test_pack_int(123456789, &[0x18, 0x07, 0x5b, 0xcd, 0x15]);
        test_pack_int(i64::min_value(), &[0x0c, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        test_pack_int(i64::max_value(), &[0x1c, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn pack_uint_testcases() {
        test_pack_uint(0, &[0x14]);
        test_pack_uint(1, &[0x15, 0x01]);
        test_pack_uint(2, &[0x15, 0x02]);
        test_pack_uint(123, &[0x15, 0x7b]);
        test_pack_uint(123456789, &[0x18, 0x07, 0x5b, 0xcd, 0x15]);
        test_pack_uint(i64::max_value() as u64, &[0x1c, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        test_pack_uint(u64::max_value(), &[0x1c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe])
        // Apple's fdb.tuple encodes this slightly less normalized, as [0x1d, 0x08, 0xff, ...]
        // Why? I don't know. It'll still decode this more compact representation
        test_pack_uint(u64::max_value(), &[0x1c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    }
}
