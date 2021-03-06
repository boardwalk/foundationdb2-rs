use foundationdb_sys as fdb;
use std::borrow::Cow;
use std::ffi::CStr;
use std::mem::size_of;
use std::os::raw::{c_char, c_int, c_void};
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

#[repr(packed)]
struct RawKeyValue {
    key: *const c_void,
    key_len: c_int,
    value: *const c_void,
    value_len: c_int,
}

pub struct KeyValue<'a> {
    kv: &'a fdb::FDBKeyValue,
}

#[allow(clippy::cast_ptr_alignment)]
impl<'a> KeyValue<'a> {
    pub fn key(&self) -> &[u8] {
        let kv = self.kv as *const _ as *const RawKeyValue;
        unsafe { slice::from_raw_parts((*kv).key as *const _, (*kv).key_len as usize) }
    }

    pub fn value(&self) -> &[u8] {
        let kv = self.kv as *const _ as *const RawKeyValue;
        unsafe { slice::from_raw_parts((*kv).value as *const _, (*kv).value_len as usize) }
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
        // TODO: This assert should be in a test
        debug_assert_eq!(size_of::<RawKeyValue>(), size_of::<fdb::FDBKeyValue>());
        KeyValue {
            kv: unsafe { &*self.kv.add(index) },
        }
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.count != 0
    }

    pub fn more(&self) -> bool {
        self.more != 0
    }

    pub fn iter(&self) -> KeyValueArrayIter {
        KeyValueArrayIter { arr: self, i: 0 }
    }
}

impl Drop for KeyValueArray {
    fn drop(&mut self) {
        unsafe { fdb::fdb_future_destroy(self.fut) };
    }
}

pub struct KeyValueArrayIter<'a> {
    arr: &'a KeyValueArray,
    i: usize,
}

impl<'a> Iterator for KeyValueArrayIter<'a> {
    type Item = KeyValue<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.arr.len() {
            let i = self.i;
            self.i += 1;
            Some(self.arr.get(i))
        } else {
            None
        }
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

    pub fn is_empty(&self) -> bool {
        self.count != 0
    }
}

impl Drop for StringArray {
    fn drop(&mut self) {
        unsafe { fdb::fdb_future_destroy(self.fut) };
    }
}
