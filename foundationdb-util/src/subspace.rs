use crate::tuple::{Pack, Tuple, Unpack, UnpackError};

pub struct Subspace {
    prefix_bytes: Vec<u8>,
}

impl Subspace {
    pub fn new<T: Tuple + Pack>(prefix: &T) -> Self {
        let mut prefix_bytes = Vec::new();
        prefix.pack(&mut prefix_bytes, false);
        Subspace { prefix_bytes }
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
        inp.len() >= self.prefix_bytes.len()
            && &inp[..self.prefix_bytes.len()] == self.prefix_bytes.as_slice()
    }

    pub fn subspace<T: Tuple + Pack>(&self, tuple: &T) -> Self {
        let mut prefix_bytes = self.prefix_bytes.clone();
        tuple.pack(&mut prefix_bytes, false);
        Subspace { prefix_bytes }
    }
}

impl AsRef<[u8]> for Subspace {
    fn as_ref(&self) -> &[u8] {
        self.prefix_bytes.as_slice()
    }
}

#[cfg(test)]
mod test {
    use super::Subspace;
    use crate::tuple::UnpackError;

    #[test]
    fn subspace() {
        let s1 = Subspace::new(&("entities",));
        let s2 = Subspace::new(&("entities", 356));
        let s3 = Subspace::new(&("entities", 789));

        let k1 = s1.pack(&(356, "state"));
        assert!(s1.contains(&k1));
        assert!(s2.contains(&k1));
        assert!(!s3.contains(&k1));

        let (eid, field): (i32, String) = s1.unpack(&k1).unwrap();
        assert_eq!(eid, 356);
        assert_eq!(field, "state");

        let (field,): (String,) = s2.unpack(&k1).unwrap();
        assert_eq!(field, "state");

        let r: Result<(String,), _> = s3.unpack(&k1);
        assert_eq!(r, Err(UnpackError::MissingPrefix));

        let s4 = s1.subspace(&(356,));
        assert_eq!(s2.key(), s4.key());
    }
}
