use crate::tuple::{Tuple, Pack, Unpack, UnpackError};

pub struct Subspace {
    prefix_bytes: Vec<u8>,
}

impl Subspace {
    pub fn new<T: Tuple + Pack>(prefix: &T) -> Self {
        let mut prefix_bytes = Vec::new();
        prefix.pack(&mut prefix_bytes, false);
        Subspace { prefix_bytes }
    }

    pub fn key(&self) -> &[u8] {
        self.prefix_bytes.as_slice()
    }

    pub fn pack<T: Tuple + Pack>(&self, tuple: &T) -> Vec<u8> {
        let mut bytes = self.prefix_bytes.clone();
        tuple.pack(&mut bytes, false);
        bytes
    }

    pub fn unpack<T: Tuple + Unpack>(&self, inp: &[u8]) -> Result<T, UnpackError> {
        if inp.len() < self.prefix_bytes.len() {
            return Err(UnpackError::MissingPrefix);
        }

        let (prefix, inp) = inp.split_at(self.prefix_bytes.len());

        if prefix != self.prefix_bytes.as_slice() {
            return Err(UnpackError::MissingPrefix);
        }

        match T::unpack(inp, false) {
            Ok((tuple, rest)) => {
                if rest.is_empty() {
                    Ok(tuple)
                } else {
                    Err(UnpackError::TrailingData)
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn range<T: Tuple + Pack>(&self, tuple: &T) -> (Vec<u8>, Vec<u8>) {
        let bytes = self.pack(tuple);
        let mut begin = bytes.clone();
        begin.push(0x00);
        let mut end = bytes.clone();
        end.push(0xFF);
        (begin, end)
    }

    pub fn contains(&self, inp: &[u8]) -> bool {
        inp.len() >= self.prefix_bytes.len() &&
            &inp[..self.prefix_bytes.len()] == self.prefix_bytes.as_slice()
    }

    pub fn subspace<T: Tuple + Pack>(&self, tuple: &T) -> Self {
        let mut prefix_bytes = self.prefix_bytes.clone();
        tuple.pack(&mut prefix_bytes, false);
        Subspace { prefix_bytes }
    }
}
