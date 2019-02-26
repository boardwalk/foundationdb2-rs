use crate::future::{Future, Error, Value};
use foundationdb_sys as fdb;
use std::os::raw::c_int;

pub async fn retry(f: impl Fn() -> Transaction) -> Result<(), Error> {
    loop {
        let tran = f();
        match await!(tran.commit()) {
            Ok(_) => return Ok(()),
            Err(err) => {
                let fut = unsafe { fdb::fdb_transaction_on_error(tran.tran, err.err()) };
                let _ = await!(Future::new(fut))?;
            }
        }
    }
}

/*
 * Missing:
 * fdb_transaction_set_option
 * fdb_transaction_set_read_version
 * fdb_transaction_get_read_version
 * fdb_transaction_get_key
 * fdb_transaction_get_addresses_for_key
 * fdb_transaction_get_range
 * fdb_transaction_atomic_op
 * fdb_transaction_get_committed_version
 * fdb_transaction_get_versionstamp
 * fdb_transaction_add_conflict_range
 */

pub struct Transaction {
    pub(crate) tran: *mut fdb::FDBTransaction,
}

impl Transaction {
    pub async fn get<'a>(&'a self, key: &'a [u8], snapshot: bool) -> Result<Option<Value>, Error> {
        let fut = unsafe { fdb::fdb_transaction_get(self.tran, key.as_ptr(), key.len() as c_int, snapshot as fdb::fdb_bool_t) };
        let rfut = await!(Future::new(fut))?;
        rfut.into_value()
    }

    pub fn set(&self, key: &[u8], value: &[u8]) {
        unsafe { fdb::fdb_transaction_set(self.tran, key.as_ptr(), key.len() as c_int, value.as_ptr(), value.len() as c_int) };
    }

    pub fn clear(&self, key: &[u8]) {
        unsafe { fdb::fdb_transaction_clear(self.tran, key.as_ptr(), key.len() as c_int) };
    }

    pub fn clear_range(&self, begin_key: &[u8], end_key: &[u8]) {
        unsafe { fdb::fdb_transaction_clear_range(self.tran, begin_key.as_ptr(), begin_key.len() as c_int, end_key.as_ptr(), end_key.len() as c_int) };
    }

    pub async fn commit(&self) -> Result<(), Error> {
        let fut = unsafe { fdb::fdb_transaction_commit(self.tran) };
        let _rfut = await!(Future::new(fut))?;
        Ok(())
    }

    pub async fn watch<'a>(&'a self, key: &'a [u8]) -> Result<(), Error> {
        let fut = unsafe { fdb::fdb_transaction_watch(self.tran, key.as_ptr(), key.len() as c_int) };
        let _ = await!(Future::new(fut))?;
        Ok(())
    }

    pub fn reset(&self) {
        unsafe { fdb::fdb_transaction_reset(self.tran) };
    }

    pub fn cancel(&self) {
        unsafe { fdb::fdb_transaction_cancel(self.tran) };
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        unsafe { fdb::fdb_transaction_destroy(self.tran) };
    }
}
