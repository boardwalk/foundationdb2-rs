use crate::tuple::{TuplePack, TupleUnpack, UnpackError};
use byteorder::{ByteOrder, BigEndian};

const NEG_INT_START: u8 = 0x0b;
const INT_ZERO_CODE: u8 = 0x14;
const POS_INT_END: u8 = 0x1d;

fn pack_int(inp: i64, out: &mut Vec<u8>) {
    let nbytes = ((64 - inp.wrapping_abs().leading_zeros() + 7) / 8) as usize;
    if inp >= 0 {
        out.push(INT_ZERO_CODE + nbytes as u8);

        let mut buf = [0; 8];
        BigEndian::write_i64(&mut buf, inp);
        out.extend_from_slice(&buf[8 - nbytes..]);
    } else {
        out.push(INT_ZERO_CODE - nbytes as u8);

        let max_value = 1i64.checked_shl(nbytes as u32 * 8).unwrap_or(0).wrapping_sub(1);
        let mut buf = [0; 8];
        BigEndian::write_i64(&mut buf, max_value.wrapping_add(inp));
        out.extend_from_slice(&buf[8 - nbytes..]);
    }
}

fn unpack_int(inp: &[u8]) -> Result<(i64, &[u8]), UnpackError> {
    if let Some((&code, inp)) = inp.split_first() {
        if code >= INT_ZERO_CODE && code <= POS_INT_END {
            let nbytes = (code - INT_ZERO_CODE) as usize;
            if inp.len() < nbytes {
                return Err(UnpackError::OutOfData);
            }

            let mut buf = [0; 8];
            buf[8 - nbytes..].copy_from_slice(&inp[..nbytes]);
            let out = BigEndian::read_i64(&buf);

            Ok((out, &inp[nbytes..]))
        } else if code > NEG_INT_START && code < INT_ZERO_CODE {
            let nbytes = (INT_ZERO_CODE - code) as usize;
            if inp.len() < nbytes {
                return Err(UnpackError::OutOfData);
            }

            let max_value = 1i64.checked_shl(nbytes as u32 * 8).unwrap_or(0).wrapping_sub(1);
            let mut buf = [0; 8];
            buf[8 - nbytes..].copy_from_slice(&inp[..nbytes]);
            let out = BigEndian::read_i64(&buf).wrapping_sub(max_value);

            Ok((out, &inp[nbytes..]))
        } else {
            Err(UnpackError::WrongCode)
        }
    } else {
        Err(UnpackError::OutOfData)
    }
}

#[cfg(test)]
mod test {
    use rand::random;
    use super::{pack_int, unpack_int};

    fn test_one(in_val: i64, buf: &mut Vec<u8>) {
        buf.clear();
        pack_int(in_val, buf);
        let (out_val, rest) = unpack_int(buf).unwrap();
        assert_eq!(in_val, out_val);
        assert!(rest.is_empty());
    }

    #[test]
    fn pack_unpack_int() {
        let mut buf = Vec::new();
        test_one(i64::min_value(), &mut buf);
        test_one(i64::max_value(), &mut buf);
        test_one(0, &mut buf);
        for _ in 0..100000 {
            let in_val = random::<i64>();
            test_one(in_val, &mut buf);
        }
    }
}
