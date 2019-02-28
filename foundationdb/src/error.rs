use foundationdb_sys as fdb;
use std::error;
use std::ffi::CStr;
use std::fmt;

pub struct Error {
    pub(crate) err: fdb::fdb_error_t,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let err_str = unsafe { CStr::from_ptr(fdb::fdb_get_error(self.err)) };
        write!(f, "[{}] {}", self.err, err_str.to_string_lossy())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let err_str = unsafe { CStr::from_ptr(fdb::fdb_get_error(self.err)) };
        write!(f, "{}", err_str.to_string_lossy())
    }
}

impl error::Error for Error {}
