use std::string::FromUtf8Error;
use byteorder::{ByteOrder, BigEndian};

// const NULL_CODE: u8 = 0x00;
const BYTES_CODE: u8 = 0x01;
const STRING_CODE: u8 = 0x02;
const NESTED_CODE: u8 = 0x05;
const INT_ZERO_CODE: u8 = 0x14;
const POS_INT_END: u8 = 0x1d;
const NEG_INT_START: u8 = 0x0b;
// const FLOAT_CODE: u8 = 0x20;
// const DOUBLE_CODE: u8 = 0x21;
const FALSE_CODE: u8 = 0x26;
const TRUE_CODE: u8 = 0x27;
// const UUID_CODE: u8 = 0x30;
// const VERSIONSTAMP_CODE: u8 = 0x33;

#[derive(Debug)]
pub enum UnpackError {
    WrongCode,
    OutOfData,
    BadEncoding(FromUtf8Error),
}

pub trait TuplePack {
    fn encode(&self, out: &mut Vec<u8>, nested: bool);
}

pub trait TupleUnpack: Sized {
    fn decode(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError>;
}

fn expect(inp: &[u8], expected: u8) -> Result<&[u8], UnpackError> {
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

fn write_bytes(data: &[u8], out: &mut Vec<u8>) {
    for b in data {
        out.push(*b);
        if *b == 0x00 {
            out.push(0xFF);
        }
    }
}

fn read_bytes(inp: &[u8]) -> (Vec<u8>, &[u8]) {
    let mut out = Vec::new();
    let mut i = 0;
    while i < inp.len() {
        if inp[i] == 0x00 {
            if i + 1 < inp.len() && inp[i + 1] == 0xFF {
                // escaped null
                out.push(inp[i]);
                i += 2;
            } else {
                // end of tuple element
                i += 1;
                break;
            }
        } else {
            out.push(inp[i]);
            i += 1;
        }
    }

    (out, &inp[i..])
}

impl TuplePack for [u8] {
    fn encode(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(BYTES_CODE);
        write_bytes(self, out);
    }
}

impl TupleUnpack for Vec<u8> {
    fn decode(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = expect(inp, BYTES_CODE)?;
        let (vec, inp) = read_bytes(inp);
        Ok((vec, inp))
    }
}

impl TuplePack for str {
    fn encode(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(STRING_CODE);
        write_bytes(self.as_bytes(), out);
    }
}

impl TupleUnpack for String {
    fn decode(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = expect(inp, STRING_CODE)?;
        let (vec, inp) = read_bytes(inp);
        match String::from_utf8(vec) {
            Ok(s) => Ok((s, inp)),
            Err(err) => Err(UnpackError::BadEncoding(err)),
        }
    }
}

impl TuplePack for bool {
    fn encode(&self, key: &mut Vec<u8>, _nested: bool) {
        key.push(if *self { TRUE_CODE } else { FALSE_CODE });
    }
}

impl TupleUnpack for bool {
    fn decode(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        if let Some((&code, inp)) = inp.split_first() {
            if code == TRUE_CODE {
                Ok((true, inp))
            } else if code == FALSE_CODE {
                Ok((false, inp))
            } else {
                Err(UnpackError::WrongCode)
            }
        } else {
            Err(UnpackError::OutOfData)
        }
    }
}

impl<T1, T2> TuplePack for (T1, T2)
where
    T1: TuplePack,
    T2: TuplePack,
{
    fn encode(&self, out: &mut Vec<u8>, nested: bool) {
        let (v1, v2) = self;
        if nested { out.push(NESTED_CODE);  }
        T1::encode(v1, out, true);
        T2::encode(v2, out, true);
        if nested { out.push(0x00); }
    }
}

impl<T1, T2> TupleUnpack for (T1, T2)
where
    T1: TupleUnpack,
    T2: TupleUnpack,
{
    fn decode(inp: &[u8], nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = if nested { expect(inp, NESTED_CODE)? } else { inp };
        let (v1, inp) = T1::decode(inp, true)?;
        let (v2, inp) = T2::decode(inp, true)?;
        let inp = if nested { expect(inp, 0x00)? } else { inp };
        Ok(((v1, v2), inp))
    }
}

impl TuplePack for i64 {
    fn encode(&self, out: &mut Vec<u8>, _nested: bool) {
        if *self == 0 {
            out.push(INT_ZERO_CODE);
        } else {
            let mul = if *self > 0 { 1 } else { -1 };
            let pos_val = (*self * mul) as u64;
            let nbytes = (64 - pos_val.leading_zeros() + 7) / 8;

            let code = INT_ZERO_CODE as i64 + nbytes as i64 * mul;
            out.push(code as u8);

            let mut buf = [0; 8];
            BigEndian::write_u64(&mut buf, pos_val);
            out.extend_from_slice(&buf[8 - nbytes as usize..]);
        }
    }
}

impl TupleUnpack for i64 {
    fn decode(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        if let Some((&code, inp)) = inp.split_first() {
            if code == INT_ZERO_CODE {
                Ok((0, inp))
            } else if code >= NEG_INT_START && code <= POS_INT_END {
                let mul = if code > INT_ZERO_CODE { 1 } else { -1 };
                let nbytes = ((code as i64 - INT_ZERO_CODE as i64) * mul) as usize;
                if inp.len() < nbytes {
                    return Err(UnpackError::OutOfData)
                }

                let mut buf = [0; 8];
                buf[8 - nbytes..].copy_from_slice(&inp[..nbytes]);
                let pos_val = BigEndian::read_u64(&buf);
                let sig_val = (pos_val as i64) * mul;

                Ok((sig_val, &inp[nbytes..]))
            } else {
                Err(UnpackError::WrongCode)
            }
        } else {
            Err(UnpackError::OutOfData)
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::random;
    use std::fmt::Debug;
    use super::{TuplePack, TupleUnpack};

    fn test_pack_unpack<Tin, Tout>(in_val: Tin, buf: &mut Vec<u8>)
    where
        Tin: TuplePack + Debug,
        Tout: TupleUnpack + Debug,
        Tin: PartialEq<Tout>,
    {
        buf.clear();
        TuplePack::encode(&in_val, buf, false);
        let (out_val, rest) = <Tout as TupleUnpack>::decode(&buf, false).unwrap();
        assert_eq!(in_val, out_val);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_i64() {
        let mut buf = Vec::new();
        for _ in 0..10000 {
            test_pack_unpack(random::<i64>(), &mut buf);
        }
    }
}