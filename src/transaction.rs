use crate::future::{Error, Future, Key, KeyValueArray, Value};
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

pub struct KeySelector<'a> {
    key: &'a [u8],
    equal: bool,
    offset: i32,
}

pub enum StreamingMode {
    Iterator,
    Small,
    Medium,
    Large,
    Serial,
    WantAll,
    Exact,
}

impl StreamingMode {
    fn as_c_enum(&self) -> fdb::FDBStreamingMode {
        use StreamingMode::*;
        match self {
            Iterator => fdb::FDBStreamingMode_FDB_STREAMING_MODE_ITERATOR,
            Small => fdb::FDBStreamingMode_FDB_STREAMING_MODE_SMALL,
            Medium => fdb::FDBStreamingMode_FDB_STREAMING_MODE_MEDIUM,
            Large => fdb::FDBStreamingMode_FDB_STREAMING_MODE_LARGE,
            Serial => fdb::FDBStreamingMode_FDB_STREAMING_MODE_SERIAL,
            WantAll => fdb::FDBStreamingMode_FDB_STREAMING_MODE_WANT_ALL,
            Exact => fdb::FDBStreamingMode_FDB_STREAMING_MODE_EXACT,
        }
    }
}

pub struct GetRangeOpt<'a> {
    begin_selector: KeySelector<'a>,
    end_selector: KeySelector<'a>,
    limit: i32,
    target_bytes: i32,
    mode: StreamingMode,
    iteration: i32,
    snapshot: bool,
    reverse: bool,
}

pub enum MutationType {
    Add,
    And,
    Or,
    Xor,
    Max,
    ByteMax,
    Min,
    ByteMin,
    SetVersionstampedKey,
}

impl MutationType {
    fn as_c_enum(&self) -> fdb::FDBMutationType {
        use MutationType::*;
        match self {
            Add => fdb::FDBMutationType_FDB_MUTATION_TYPE_ADD,
            And => fdb::FDBMutationType_FDB_MUTATION_TYPE_AND,
            Or => fdb::FDBMutationType_FDB_MUTATION_TYPE_OR,
            Xor => fdb::FDBMutationType_FDB_MUTATION_TYPE_XOR,
            Max => fdb::FDBMutationType_FDB_MUTATION_TYPE_MAX,
            ByteMax => fdb::FDBMutationType_FDB_MUTATION_TYPE_BYTE_MAX,
            Min => fdb::FDBMutationType_FDB_MUTATION_TYPE_MIN,
            ByteMin => fdb::FDBMutationType_FDB_MUTATION_TYPE_BYTE_MIN,
            SetVersionstampedKey => fdb::FDBMutationType_FDB_MUTATION_TYPE_SET_VERSIONSTAMPED_KEY,
        }
    }
}

/*
 * Missing:
 * fdb_transaction_set_option
 * fdb_transaction_set_read_version
 * fdb_transaction_get_read_version
 * fdb_transaction_get_addresses_for_key
 * fdb_transaction_get_committed_version
 * fdb_transaction_get_versionstamp
 * fdb_transaction_add_conflict_range
 */

pub struct Transaction {
    pub(crate) tran: *mut fdb::FDBTransaction,
}

impl Transaction {
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

    pub async fn get_key<'a>(&'a self, selector: KeySelector<'a>, snapshot: bool) -> Result<Key, Error> {
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

    pub async fn get_range<'a>(&'a self, opt: GetRangeOpt<'a>) -> Result<KeyValueArray, Error> {
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
        unsafe {
            fdb::fdb_transaction_clear(
                self.tran,
                key.as_ptr(),
                key.len() as c_int,
            )
        };
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

    pub async fn commit(&self) -> Result<(), Error> {
        let fut = unsafe { fdb::fdb_transaction_commit(self.tran) };
        let _rfut = await!(Future::new(fut))?;
        Ok(())
    }

    pub async fn watch<'a>(&'a self, key: &'a [u8]) -> Result<(), Error> {
        let fut = unsafe {
            fdb::fdb_transaction_watch(
                self.tran,
                key.as_ptr(),
                key.len() as c_int,
            )
        };
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
