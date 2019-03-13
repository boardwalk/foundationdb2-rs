use crate::error::Error;
use crate::outputs::{Key, KeyValueArray, StringArray, Value};
use foundationdb_sys as fdb;
use std::mem::replace;
use std::ptr::{null, null_mut};

/*
 * FutureReady
 */

pub struct FutureReady {
    fut: *mut fdb::FDBFuture,
}

impl FutureReady {
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

impl Drop for FutureReady {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}
