use crate::error::Error;
use crate::future::{Future, Key, KeyValueArray, StringArray, Value};
use foundationdb_sys as fdb;
use std::mem::replace;
use std::os::raw::c_int;
use std::ptr;

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

#[derive(Clone, Copy)]
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
    fn as_c_enum(self) -> fdb::FDBMutationType {
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

#[derive(Clone, Copy)]
pub enum ConflictRangeType {
    Read,
    Write,
}

impl ConflictRangeType {
    fn as_c_enum(self) -> fdb::FDBConflictRangeType {
        use ConflictRangeType::*;
        match self {
            Read => fdb::FDBConflictRangeType_FDB_CONFLICT_RANGE_TYPE_READ,
            Write => fdb::FDBConflictRangeType_FDB_CONFLICT_RANGE_TYPE_WRITE,
        }
    }
}

// TODO We could generate this from fdb.options
#[derive(Clone, Copy)]
pub enum TransactionOption {
    CausalWriteRisky,
    CausalReadRisky,
    CausalReadDisable,
    NextWriteNoWriteConflictRange,
    ReadYourWritesDisable,
    #[deprecated] ReadAheadDisable,
    DurabilityDatacenter,
    DurabilityRisky,
    #[deprecated] DurabilityDevNullIsWebScale,
    PrioritySystemImmediate,
    PriorityBatch,
    InitializeNewDatabase,
    AccessSystemKeys,
    ReadSystemKeys,
    DebugRetryLogging,
    TransactionLoggingEnable,
    Timeout,
    RetryLimit,
    MaxRetryDelay,
    SnapshotRywEnable,
    SnapshotRywDisable,
    LockAware,
    UsedDuringCommitProtectionDisable,
    ReadLockAware,
}

impl TransactionOption {
    fn as_c_enum(self) -> fdb::FDBTransactionOption {
        use TransactionOption::*;
        match self {
            CausalWriteRisky => fdb::FDBTransactionOption_FDB_TR_OPTION_CAUSAL_WRITE_RISKY,
            CausalReadRisky => fdb::FDBTransactionOption_FDB_TR_OPTION_CAUSAL_READ_RISKY,
            CausalReadDisable => fdb::FDBTransactionOption_FDB_TR_OPTION_CAUSAL_READ_DISABLE,
            NextWriteNoWriteConflictRange => fdb::FDBTransactionOption_FDB_TR_OPTION_NEXT_WRITE_NO_WRITE_CONFLICT_RANGE,
            ReadYourWritesDisable => fdb::FDBTransactionOption_FDB_TR_OPTION_READ_YOUR_WRITES_DISABLE,
            ReadAheadDisable => fdb::FDBTransactionOption_FDB_TR_OPTION_READ_AHEAD_DISABLE,
            DurabilityDatacenter => fdb::FDBTransactionOption_FDB_TR_OPTION_DURABILITY_DATACENTER,
            DurabilityRisky => fdb::FDBTransactionOption_FDB_TR_OPTION_DURABILITY_RISKY,
            DurabilityDevNullIsWebScale => fdb::FDBTransactionOption_FDB_TR_OPTION_DURABILITY_DEV_NULL_IS_WEB_SCALE,
            PrioritySystemImmediate => fdb::FDBTransactionOption_FDB_TR_OPTION_PRIORITY_SYSTEM_IMMEDIATE,
            PriorityBatch => fdb::FDBTransactionOption_FDB_TR_OPTION_PRIORITY_BATCH,
            InitializeNewDatabase => fdb::FDBTransactionOption_FDB_TR_OPTION_INITIALIZE_NEW_DATABASE,
            AccessSystemKeys => fdb::FDBTransactionOption_FDB_TR_OPTION_ACCESS_SYSTEM_KEYS,
            ReadSystemKeys => fdb::FDBTransactionOption_FDB_TR_OPTION_READ_SYSTEM_KEYS,
            DebugRetryLogging => fdb::FDBTransactionOption_FDB_TR_OPTION_DEBUG_RETRY_LOGGING,
            TransactionLoggingEnable => fdb::FDBTransactionOption_FDB_TR_OPTION_TRANSACTION_LOGGING_ENABLE,
            Timeout => fdb::FDBTransactionOption_FDB_TR_OPTION_TIMEOUT,
            RetryLimit => fdb::FDBTransactionOption_FDB_TR_OPTION_RETRY_LIMIT,
            MaxRetryDelay => fdb::FDBTransactionOption_FDB_TR_OPTION_MAX_RETRY_DELAY,
            SnapshotRywEnable => fdb::FDBTransactionOption_FDB_TR_OPTION_SNAPSHOT_RYW_ENABLE,
            SnapshotRywDisable => fdb::FDBTransactionOption_FDB_TR_OPTION_SNAPSHOT_RYW_DISABLE,
            LockAware => fdb::FDBTransactionOption_FDB_TR_OPTION_LOCK_AWARE,
            UsedDuringCommitProtectionDisable => fdb::FDBTransactionOption_FDB_TR_OPTION_USED_DURING_COMMIT_PROTECTION_DISABLE,
            ReadLockAware => fdb::FDBTransactionOption_FDB_TR_OPTION_READ_LOCK_AWARE,
        }
    }
}

/*
 * Transaction
 */

pub struct Transaction {
    pub(crate) tran: *mut fdb::FDBTransaction,
}

impl Transaction {
    pub fn set_option(&self, option: TransactionOption, value: &[u8]) {
        unsafe {
            fdb::fdb_transaction_set_option(
                self.tran,
                option.as_c_enum(),
                value.as_ptr(),
                value.len() as c_int,
            )
        };
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

    pub async fn commit(mut self) -> Result<CommittedTransaction, FailedTransaction> {
        let fut = unsafe { fdb::fdb_transaction_commit(self.tran) };
        match await!(Future::new(fut)) {
            Ok(_) => Ok(CommittedTransaction {
                tran: replace(&mut self.tran, ptr::null_mut()),
            }),
            Err(err) => Err(FailedTransaction {
                tran: replace(&mut self.tran, ptr::null_mut()),
                err: err.err,
            }),
        }
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

    pub fn set_read_version(&self, version: i64) {
        unsafe { fdb::fdb_transaction_set_read_version(self.tran, version) };
    }

    pub async fn get_read_version(&self) -> Result<i64, Error> {
        let fut = unsafe {
            fdb::fdb_transaction_get_read_version(self.tran)
        };
        let rfut = await!(Future::new(fut))?;
        rfut.into_version()
    }

    pub async fn get_addresses_for_key<'a>(&'a self, key: &'a [u8]) -> Result<StringArray, Error> {
        let fut = unsafe {
            fdb::fdb_transaction_get_addresses_for_key(
                self.tran,
                key.as_ptr(),
                key.len() as c_int,
            )
        };
        let rfut = await!(Future::new(fut))?;
        rfut.into_string_array()
    }

    pub async fn get_versionstamp(&self) -> Result<Key, Error> {
        let fut = unsafe {
            fdb::fdb_transaction_get_versionstamp(self.tran)
        };
        let rfut = await!(Future::new(fut))?;
        rfut.into_key()
    }

    pub fn add_conflict_range<'a>(&'a self, begin_key: &'a [u8], end_key: &'a [u8], range_type: ConflictRangeType) -> Result<(), Error> {
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
            Ok(_) => {
                Ok(Transaction {
                    tran: replace(&mut self.tran, ptr::null_mut()),
                })
            }
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
