use foundationdb_sys as fdb;
use futures;
use futures::task::{AtomicWaker, Waker};
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::mem::replace;
use std::os::raw::c_int;
use std::pin::Pin;
use std::ptr;
use std::slice;

/*
 * Future
 */

pub struct Future {
    fut: *mut fdb::FDBFuture,
    waker: AtomicWaker,
    registered: bool,
}

impl Future {
    pub fn new(fut: *mut fdb::FDBFuture) -> Self {
        Self {
            fut,
            waker: AtomicWaker::new(),
            registered: false,
        }
    }
}

impl futures::Future for Future {
    type Output = Result<ReadyFuture, Error>;

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> futures::Poll<Self::Output> {
        debug_assert!(!self.fut.is_null());

        if !self.registered {
            self.waker.register(waker);

            unsafe {
                fdb::fdb_future_set_callback(
                    self.fut,
                    Some(fdb_future_callback),
                    &self.waker as *const _ as *mut _,
                );
            }

            self.registered = true;
            return futures::Poll::Pending;
        }

        let ready = unsafe { fdb::fdb_future_is_ready(self.fut) };
        if ready == 0 {
            return futures::Poll::Pending;
        }

        let err = unsafe { fdb::fdb_future_get_error(self.fut) };
        if err != 0 {
            return futures::Poll::Ready(Err(Error::new(err)));
        }

        let res = ReadyFuture::new(replace(&mut self.fut, ptr::null_mut()));
        futures::Poll::Ready(Ok(res))
    }
}

impl Drop for Future {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}

extern "C" fn fdb_future_callback(
    _fut: *mut fdb::FDBFuture,
    callback_parameter: *mut ::std::os::raw::c_void,
) {
    let awaker: *const AtomicWaker = callback_parameter as *const _;
    unsafe {
        (*awaker).wake();
    }
}

/*
 * Value
 */

pub struct Value {
    fut: *mut fdb::FDBFuture,
    val: *const u8,
    val_len: c_int,
}

impl AsRef<[u8]> for Value {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.val, self.val_len as usize) }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe { fdb::fdb_future_destroy(self.fut) };
    }
}

/*
 * ReadyFuture
 */

pub struct ReadyFuture {
    fut: *mut fdb::FDBFuture,
}

impl ReadyFuture {
    pub fn new(fut: *mut fdb::FDBFuture) -> Self {
        Self { fut }
    }

    pub fn into_cluster(self) -> Result<*mut fdb::FDBCluster, Error> {
        let mut val = ptr::null_mut();
        bail!(unsafe { fdb::fdb_future_get_cluster(self.fut, &mut val) });
        Ok(val)
    }

    pub fn into_database(self) -> Result<*mut fdb::FDBDatabase, Error> {
        let mut val = ptr::null_mut();
        bail!(unsafe { fdb::fdb_future_get_database(self.fut, &mut val) });
        Ok(val)
    }

    pub fn into_value(mut self) -> Result<Option<Value>, Error> {
        let mut present = 0;
        let mut val = ptr::null();
        let mut val_len = 0;
        bail!(unsafe { fdb::fdb_future_get_value(self.fut, &mut present, &mut val, &mut val_len) });
        if present != 0 {
            Ok(Some(Value {
                fut: replace(&mut self.fut, ptr::null_mut()),
                val,
                val_len,
            }))
        } else {
            Ok(None)
        }
    }
}

impl Drop for ReadyFuture {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}

/*
 * Error
 */

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
