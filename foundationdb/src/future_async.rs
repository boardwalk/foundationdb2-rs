use crate::error::Error;
use crate::future_ready::FutureReady;
use foundationdb_sys as fdb;
use futures::task::{AtomicWaker, Waker};
use futures::{self, Poll};
use std::mem::replace;
use std::os::raw::c_void;
use std::pin::Pin;
use std::ptr::null_mut;

/*
 * FutureAsync
 */

pub struct FutureAsync {
    fut: *mut fdb::FDBFuture,
    waker: AtomicWaker,
    registered: bool,
}

impl FutureAsync {
    pub fn new(fut: *mut fdb::FDBFuture) -> Self {
        Self {
            fut,
            waker: AtomicWaker::new(),
            registered: false,
        }
    }
}

impl futures::Future for FutureAsync {
    type Output = Result<FutureReady, Error>;

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

        let res = FutureReady::new(replace(&mut self.fut, null_mut()));
        Poll::Ready(Ok(res))
    }
}

impl Drop for FutureAsync {
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
