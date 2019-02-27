use foundationdb_sys as fdb;
use std::fmt;
use std::error;
use std::ffi::CStr;

pub struct Error {
    err: fdb::fdb_error_t,
}

impl Error {
    pub fn new(err: fdb::fdb_error_t) -> Self {
        Self { err }
    }

    pub(crate) fn err(&self) -> fdb::fdb_error_t {
        self.err
    }
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
