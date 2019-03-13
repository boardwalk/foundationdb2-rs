use crate::error::Error;
use crate::future_async::FutureAsync;
use crate::outputs::{Key, KeyValueArray, StringArray, Value};
use crate::transaction::{CommittedTransaction, FailedTransaction, GetRangeOpt, KeySelector, Transaction};
use foundationdb_sys as fdb;
use futures::{future::ready, Future, FutureExt, TryFutureExt};
use std::mem::replace;
use std::os::raw::c_int;
use std::ptr::null_mut;

/*
 * Transaction
 */

impl Transaction {
    pub fn get_async<'a>(
        &'a self,
        key: &'a [u8],
        snapshot: bool,
    ) -> impl Future<Output = Result<Option<Value>, Error>> {
        let fut = unsafe {
            fdb::fdb_transaction_get(
                self.tran,
                key.as_ptr(),
                key.len() as c_int,
                snapshot as fdb::fdb_bool_t,
            )
        };
        FutureAsync::new(fut).and_then(|fut| ready(fut.into_value()))
    }

    pub fn get_key_async<'a>(
        &'a self,
        selector: KeySelector<'a>,
        snapshot: bool,
    ) -> impl Future<Output = Result<Key, Error>> {
        let fut = unsafe {
            fdb::fdb_transaction_get_key(
                self.tran,
                selector.key.as_ptr(),
                selector.key.len() as c_int,
                selector.equal as fdb::fdb_bool_t,
                selector.offset as c_int,
                snapshot as fdb::fdb_bool_t,
            )
        };
        FutureAsync::new(fut).and_then(|fut| ready(fut.into_key()))
    }

    pub fn get_range_async<'a>(
        &'a self,
        opt: &'a GetRangeOpt<'a>,
    ) -> impl Future<Output = Result<KeyValueArray, Error>> {
        let fut = self.get_range_raw(opt);
        FutureAsync::new(fut).and_then(|fut| ready(fut.into_keyvalue_array()))
    }

    pub fn commit_async(
        mut self,
    ) -> impl Future<Output = Result<CommittedTransaction, FailedTransaction>> {
        let fut = unsafe { fdb::fdb_transaction_commit(self.tran) };
        FutureAsync::new(fut).map(move |res| match res {
            Ok(_) => Ok(CommittedTransaction {
                tran: replace(&mut self.tran, null_mut()),
            }),
            Err(err) => Err(FailedTransaction {
                tran: replace(&mut self.tran, null_mut()),
                err: err.err,
            }),
        })
    }

    pub fn get_read_version_async(&self) -> impl Future<Output = Result<i64, Error>> {
        let fut = unsafe { fdb::fdb_transaction_get_read_version(self.tran) };
        FutureAsync::new(fut).and_then(|fut| ready(fut.into_version()))
    }

    pub fn watch_async<'a>(
        &'a self,
        key: &'a [u8],
    ) -> impl Future<Output = Result<(), Error>> {
        let fut =
            unsafe { fdb::fdb_transaction_watch(self.tran, key.as_ptr(), key.len() as c_int) };
        FutureAsync::new(fut).map_ok(|_| ())
    }

    pub fn get_addresses_for_key_async<'a>(
        &'a self,
        key: &'a [u8],
    ) -> impl Future<Output = Result<StringArray, Error>> {
        let fut = unsafe {
            fdb::fdb_transaction_get_addresses_for_key(self.tran, key.as_ptr(), key.len() as c_int)
        };
        FutureAsync::new(fut).and_then(|fut| ready(fut.into_string_array()))
    }

    pub fn get_versionstamp_async(&self) -> impl Future<Output = Result<Key, Error>> {
        let fut = unsafe { fdb::fdb_transaction_get_versionstamp(self.tran) };
        FutureAsync::new(fut).and_then(|fut| ready(fut.into_key()))
    }
}

/*
 * FailedTransaction
 */

impl FailedTransaction {
    pub fn on_error_async(
        mut self,
    ) -> impl Future<Output = Result<Transaction, FailedTransaction>> {
        let fut = unsafe { fdb::fdb_transaction_on_error(self.tran, self.err) };
        FutureAsync::new(fut).map(|res| match res {
            Ok(_) => Ok(Transaction {
                tran: replace(&mut self.tran, null_mut()),
            }),
            Err(err) => {
                self.err = err.err;
                Err(self)
            }
        })
    }
}
