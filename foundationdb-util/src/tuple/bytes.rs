use crate::tuple::{expect, TuplePack, TupleUnpack, UnpackError};

const BYTES_CODE: u8 = 0x01;
const STRING_CODE: u8 = 0x02;

fn pack_bytes(inp: &[u8], out: &mut Vec<u8>) {
    for &b in inp {
        out.push(b);
        if b == 0x00 {
            out.push(0xFF);
        }
    }
}

fn unpack_bytes(inp: &[u8]) -> (Vec<u8>, &[u8]) {
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
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(BYTES_CODE);
        pack_bytes(self, out);
    }
}

impl TupleUnpack for Vec<u8> {
    fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = expect(inp, BYTES_CODE)?;
        let (vec, inp) = unpack_bytes(inp);
        Ok((vec, inp))
    }
}

impl TuplePack for str {
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(STRING_CODE);
        pack_bytes(self.as_bytes(), out);
    }
}

impl TupleUnpack for String {
    fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = expect(inp, STRING_CODE)?;
        let (vec, inp) = unpack_bytes(inp);
        let s = String::from_utf8(vec)?;
        Ok((s, inp))
    }
}