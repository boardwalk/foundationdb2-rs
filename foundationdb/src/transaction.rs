use crate::error::Error;
use crate::future::Future;
use crate::options::{ConflictRangeType, MutationType, StreamingMode, TransactionOption};
use crate::outputs::{Key, KeyValueArray, StringArray, Value};
use foundationdb_sys as fdb;
use std::mem::replace;
use std::os::raw::c_int;
use std::ptr::null_mut;

pub struct KeySelector<'a> {
    pub key: &'a [u8],
    pub equal: bool,
    pub offset: i32,
}

pub struct GetRangeOpt<'a> {
    pub begin_selector: KeySelector<'a>,
    pub end_selector: KeySelector<'a>,
    pub limit: i32,
    pub target_bytes: i32,
    pub mode: StreamingMode,
    pub iteration: i32,
    pub snapshot: bool,
    pub reverse: bool,
}

/*
 * Transaction
 */

pub struct Transaction {
    pub(crate) tran: *mut fdb::FDBTransaction,
}

impl Transaction {
    pub fn set_option(&self, option: TransactionOption, value: &[u8]) -> Result<(), Error> {
        bail!(unsafe {
            fdb::fdb_transaction_set_option(
                self.tran,
                option.as_c_enum(),
                value.as_ptr(),
                value.len() as c_int,
            )
        });

        Ok(())
    }

    pub async fn get<'a>(&'a self, key: &'a [u8], snapshot: bool) -> Result<Option<Value>, Error> {
        let fut = unsafe {
            fdb::fdb_transaction_get(
                self.tran,
                key.as_ptr(),
                key.len() as c_int,
                snapshot as fdb::fdb_bool_t,
            )
        };
        let rfut = await!(Future::new(fut))?;
        rfut.into_value()
    }

    pub async fn get_key<'a>(
        &'a self,
        selector: KeySelector<'a>,
        snapshot: bool,
    ) -> Result<Key, Error> {
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
        let rfut = await!(Future::new(fut))?;
        rfut.into_key()
    }

    pub async fn get_range<'a>(&'a self, opt: &'a GetRangeOpt<'a>) -> Result<KeyValueArray, Error> {
        let fut = unsafe {
            fdb::fdb_transaction_get_range(
                self.tran,
                opt.begin_selector.key.as_ptr(),
                opt.begin_selector.key.len() as c_int,
                opt.begin_selector.equal as fdb::fdb_bool_t,
                opt.begin_selector.offset as c_int,
                opt.end_selector.key.as_ptr(),
                opt.end_selector.key.len() as c_int,
                opt.end_selector.equal as fdb::fdb_bool_t,
                opt.end_selector.offset as c_int,
                opt.limit as c_int,
                opt.target_bytes as c_int,
                opt.mode.as_c_enum(),
                opt.iteration as c_int,
                opt.snapshot as fdb::fdb_bool_t,
                opt.reverse as fdb::fdb_bool_t,
            )
        };
        let rfut = await!(Future::new(fut))?;
        rfut.into_keyvalue_array()
    }

    pub fn atomic_op(&self, key: &[u8], param: &[u8], mut_type: MutationType) {
        unsafe {
            fdb::fdb_transaction_atomic_op(
                self.tran,
                key.as_ptr(),
                key.len() as c_int,
                param.as_ptr(),
                param.len() as c_int,
                mut_type.as_c_enum(),
            )
        };
    }

    pub fn set(&self, key: &[u8], value: &[u8]) {
        unsafe {
            fdb::fdb_transaction_set(
                self.tran,
                key.as_ptr(),
                key.len() as c_int,
                value.as_ptr(),
                value.len() as c_int,
            )
        };
    }

    pub fn clear(&self, key: &[u8]) {
        unsafe { fdb::fdb_transaction_clear(self.tran, key.as_ptr(), key.len() as c_int) };
    }

    pub fn clear_range(&self, begin_key: &[u8], end_key: &[u8]) {
        unsafe {
            fdb::fdb_transaction_clear_range(
                self.tran,
                begin_key.as_ptr(),
                begin_key.len() as c_int,
                end_key.as_ptr(),
                end_key.len() as c_int,
            )
        };
    }

    pub async fn commit(mut self) -> Result<CommittedTransaction, FailedTransaction> {
        let fut = unsafe { fdb::fdb_transaction_commit(self.tran) };
        match await!(Future::new(fut)) {
            Ok(_) => Ok(CommittedTransaction {
                tran: replace(&mut self.tran, null_mut()),
            }),
            Err(err) => Err(FailedTransaction {
                tran: replace(&mut self.tran, null_mut()),
                err: err.err,
            }),
        }
    }

    pub async fn watch<'a>(&'a self, key: &'a [u8]) -> Result<(), Error> {
        let fut =
            unsafe { fdb::fdb_transaction_watch(self.tran, key.as_ptr(), key.len() as c_int) };
        let _ = await!(Future::new(fut))?;
        Ok(())
    }

    pub fn set_read_version(&self, version: i64) {
        unsafe { fdb::fdb_transaction_set_read_version(self.tran, version) };
    }

    pub async fn get_read_version(&self) -> Result<i64, Error> {
        let fut = unsafe { fdb::fdb_transaction_get_read_version(self.tran) };
        let rfut = await!(Future::new(fut))?;
        rfut.into_version()
    }

    pub async fn get_addresses_for_key<'a>(&'a self, key: &'a [u8]) -> Result<StringArray, Error> {
        let fut = unsafe {
            fdb::fdb_transaction_get_addresses_for_key(self.tran, key.as_ptr(), key.len() as c_int)
        };
        let rfut = await!(Future::new(fut))?;
        rfut.into_string_array()
    }

    pub async fn get_versionstamp(&self) -> Result<Key, Error> {
        let fut = unsafe { fdb::fdb_transaction_get_versionstamp(self.tran) };
        let rfut = await!(Future::new(fut))?;
        rfut.into_key()
    }

    pub fn add_conflict_range<'a>(
        &'a self,
        begin_key: &'a [u8],
        end_key: &'a [u8],
        range_type: ConflictRangeType,
    ) -> Result<(), Error> {
        bail!(unsafe {
            fdb::fdb_transaction_add_conflict_range(
                self.tran,
                begin_key.as_ptr(),
                begin_key.len() as c_int,
                end_key.as_ptr(),
                end_key.len() as c_int,
                range_type.as_c_enum(),
            )
        });
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
        if !self.tran.is_null() {
            unsafe { fdb::fdb_transaction_destroy(self.tran) };
        }
    }
}

/*
 * CommittedTransaction
 */

pub struct CommittedTransaction {
    tran: *mut fdb::FDBTransaction,
}

impl CommittedTransaction {
    pub fn get_committed_version(&self) -> Result<i64, Error> {
        let mut version = 0;
        bail!(unsafe { fdb::fdb_transaction_get_committed_version(self.tran, &mut version) });
        Ok(version)
    }
}

impl Drop for CommittedTransaction {
    fn drop(&mut self) {
        unsafe { fdb::fdb_transaction_destroy(self.tran) };
    }
}

/*
 * FailedTransaction
 */

pub struct FailedTransaction {
    tran: *mut fdb::FDBTransaction,
    err: fdb::fdb_error_t,
}

impl FailedTransaction {
    pub async fn on_error(mut self) -> Result<Transaction, FailedTransaction> {
        let fut = unsafe { fdb::fdb_transaction_on_error(self.tran, self.err) };
        match await!(Future::new(fut)) {
            Ok(_) => Ok(Transaction {
                tran: replace(&mut self.tran, null_mut()),
            }),
            Err(err) => {
                self.err = err.err;
                Err(self)
            }
        }
    }

    pub fn into_error(self) -> Error {
        Error { err: self.err }
    }
}

impl Drop for FailedTransaction {
    fn drop(&mut self) {
        if !self.tran.is_null() {
            unsafe { fdb::fdb_transaction_destroy(self.tran) };
        }
    }
}
