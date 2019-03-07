use crate::error::Error;
use crate::outputs::{Key, KeyValueArray, StringArray, Value};
use foundationdb_sys as fdb;
use futures::task::{AtomicWaker, Waker};
use futures::{self, Poll};
use std::mem::replace;
use std::os::raw::c_void;
use std::pin::Pin;
use std::ptr::{null, null_mut};

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

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        debug_assert!(!self.fut.is_null());

        let ready = unsafe { fdb::fdb_future_is_ready(self.fut) };
        if ready == 0 {
            self.waker.register(waker);

            if !replace(&mut self.registered, true) {
                unsafe {
                    fdb::fdb_future_set_callback(
                        self.fut,
                        Some(fdb_future_callback),
                        &self.waker as *const _ as *mut _,
                    );
                }
            }

            return Poll::Pending;
        }

        let err = unsafe { fdb::fdb_future_get_error(self.fut) };
        if err != 0 {
            return Poll::Ready(Err(Error { err }));
        }

        let res = ReadyFuture::new(replace(&mut self.fut, null_mut()));
        Poll::Ready(Ok(res))
    }
}

impl Drop for Future {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}

extern "C" fn fdb_future_callback(_fut: *mut fdb::FDBFuture, callback_parameter: *mut c_void) {
    let awaker: *const AtomicWaker = callback_parameter as *const _;
    unsafe { (*awaker).wake() };
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

    pub fn into_key(mut self) -> Result<Key, Error> {
        let mut key = null();
        let mut key_len = 0;
        bail!(unsafe { fdb::fdb_future_get_key(self.fut, &mut key, &mut key_len) });
        Ok(Key {
            fut: replace(&mut self.fut, null_mut()),
            key,
            key_len,
        })
    }

    pub fn into_value(mut self) -> Result<Option<Value>, Error> {
        let mut present = 0;
        let mut val = null();
        let mut val_len = 0;
        bail!(unsafe { fdb::fdb_future_get_value(self.fut, &mut present, &mut val, &mut val_len) });
        if present != 0 {
            Ok(Some(Value {
                fut: replace(&mut self.fut, null_mut()),
                val,
                val_len,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn into_keyvalue_array(mut self) -> Result<KeyValueArray, Error> {
        let mut kv = null();
        let mut count = 0;
        let mut more = 0;
        bail!(unsafe {
            fdb::fdb_future_get_keyvalue_array(self.fut, &mut kv, &mut count, &mut more)
        });
        Ok(KeyValueArray {
            fut: replace(&mut self.fut, null_mut()),
            kv,
            count,
            more,
        })
    }

    pub fn into_version(self) -> Result<i64, Error> {
        let mut version = 0;
        bail!(unsafe { fdb::fdb_future_get_version(self.fut, &mut version) });
        Ok(version)
    }

    pub fn into_string_array(mut self) -> Result<StringArray, Error> {
        let mut strings = null_mut();
        let mut count = 0;
        bail!(unsafe { fdb::fdb_future_get_string_array(self.fut, &mut strings, &mut count) });
        Ok(StringArray {
            fut: replace(&mut self.fut, null_mut()),
            strings,
            count,
        })
    }
}

impl Drop for ReadyFuture {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}
