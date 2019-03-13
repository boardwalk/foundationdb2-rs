use crate::error::Error;
use crate::future_ready::FutureReady;
use foundationdb_sys as fdb;
use std::mem::replace;
use std::ptr::null_mut;

/*
 * Future
 */

pub struct Future {
    fut: *mut fdb::FDBFuture,
}

impl Future {
    pub fn new(fut: *mut fdb::FDBFuture) -> Self {
        Self {
            fut,
        }
    }

    pub fn block_until_ready(mut self) -> Result<FutureReady, Error> {
        bail!(unsafe { fdb::fdb_future_block_until_ready(self.fut) });
        Ok(FutureReady::new(replace(&mut self.fut, null_mut())))
    }
}

impl Drop for Future {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}
