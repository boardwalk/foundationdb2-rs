use crate::tuple::{expect, Pack, Unpack, UnpackError};

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

impl Pack for [u8] {
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(BYTES_CODE);
        pack_bytes(self, out);
    }
}

impl Unpack for Vec<u8> {
    fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = expect(inp, BYTES_CODE)?;
        let (vec, inp) = unpack_bytes(inp);
        Ok((vec, inp))
    }
}

impl Pack for str {
    fn pack(&self, out: &mut Vec<u8>, _nested: bool) {
        out.push(STRING_CODE);
        pack_bytes(self.as_bytes(), out);
    }
}

impl Unpack for String {
    fn unpack(inp: &[u8], _nested: bool) -> Result<(Self, &[u8]), UnpackError> {
        let inp = expect(inp, STRING_CODE)?;
        let (vec, inp) = unpack_bytes(inp);
        let s = String::from_utf8(vec)?;
        Ok((s, inp))
    }
}

#[cfg(test)]
mod test {
    use crate::tuple::{Pack, Unpack};
    use rand::distributions::{Alphanumeric, Standard};
    use rand::{thread_rng, Rng};

    #[test]
    fn test_pack_unpack_bytes() {
        let mut buf = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..10000 {
            let nbytes = rng.gen_range(0, 64);
            let in_val = rng.sample_iter(&Standard).take(nbytes).collect::<Vec<u8>>();

            buf.clear();
            Pack::pack(&in_val[..], &mut buf, false);

            let (out_val, rest) = <Vec<u8> as Unpack>::unpack(&buf, false).unwrap();

            assert_eq!(in_val, out_val);
            assert!(rest.is_empty());
        }
    }

    #[test]
    fn test_pack_unpack_string() {
        let mut buf = Vec::new();
        let mut rng = thread_rng();
        for _ in 0..10000 {
            let nchars = rng.gen_range(0, 64);
            let in_val = rng
                .sample_iter(&Alphanumeric)
                .take(nchars)
                .collect::<String>();

            buf.clear();
            Pack::pack(in_val.as_str(), &mut buf, false);

            let (out_val, rest) = <String as Unpack>::unpack(&buf, false).unwrap();

            assert_eq!(in_val, out_val);
            assert!(rest.is_empty());
        }
    }
}
