use foundationdb_sys as fdb;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::slice;

/*
 * Key
 */

pub struct Key {
    pub(crate) fut: *mut fdb::FDBFuture,
    pub(crate) key: *const u8,
    pub(crate) key_len: c_int,
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.key, self.key_len as usize) }
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        unsafe { fdb::fdb_future_destroy(self.fut) };
    }
}

/*
 * Value
 */

pub struct Value {
    pub(crate) fut: *mut fdb::FDBFuture,
    pub(crate) val: *const u8,
    pub(crate) val_len: c_int,
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
 * KeyValue
 */

#[repr(C)]
struct RawKeyValue {
    key: *const u8,
    key_len: c_int,
    value: *const u8,
    value_len: c_int,
}

pub struct KeyValue<'a> {
    kv: &'a fdb::FDBKeyValue,
}

impl<'a> KeyValue<'a> {
    pub fn key(&self) -> &[u8] {
        let kv = self.kv as *const _ as *const RawKeyValue;
        unsafe { slice::from_raw_parts((*kv).key, (*kv).key_len as usize) }
    }

    pub fn value(&self) -> &[u8] {
        let kv = self.kv as *const _ as *const RawKeyValue;
        unsafe { slice::from_raw_parts((*kv).value, (*kv).value_len as usize) }
    }
}

/*
 * KeyValueArray
 */

pub struct KeyValueArray {
    pub(crate) fut: *mut fdb::FDBFuture,
    pub(crate) kv: *const fdb::FDBKeyValue,
    pub(crate) count: c_int,
    pub(crate) more: fdb::fdb_bool_t,
}

impl KeyValueArray {
    pub fn get(&self, index: usize) -> KeyValue {
        KeyValue {
            kv: unsafe { &*self.kv.add(index) }
        }
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn more(&self) -> bool {
        self.more != 0
    }
}

impl Drop for KeyValueArray {
    fn drop(&mut self) {
        unsafe { fdb::fdb_future_destroy(self.fut) };
    }
}

/*
 * StringArray
 */

pub struct StringArray {
    pub(crate) fut: *mut fdb::FDBFuture,
    pub(crate) strings: *mut *const c_char,
    pub(crate) count: c_int,
}

impl StringArray {
    pub fn get(&self, index: usize) -> Cow<str> {
        let s = unsafe { CStr::from_ptr(*self.strings.add(index)) };
        s.to_string_lossy()
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }
}

impl Drop for StringArray {
    fn drop(&mut self) {
        unsafe { fdb::fdb_future_destroy(self.fut) };
    }
}
