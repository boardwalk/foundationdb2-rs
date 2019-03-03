use foundationdb_sys as fdb;
use std::error;
use std::ffi::CStr;
use std::fmt::{self, Debug, Display, Formatter};

pub struct Error {
    pub(crate) err: fdb::fdb_error_t,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let err_str = unsafe { CStr::from_ptr(fdb::fdb_get_error(self.err)) };
        write!(f, "[{}] {}", self.err, err_str.to_string_lossy())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let err_str = unsafe { CStr::from_ptr(fdb::fdb_get_error(self.err)) };
        write!(f, "{}", err_str.to_string_lossy())
    }
}

impl error::Error for Error {}
