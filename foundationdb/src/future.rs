use crate::error::Error;
use foundationdb_sys as fdb;
use futures;
use futures::task::{AtomicWaker, Waker};
use std::borrow::Cow;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::replace;
use std::os::raw::{c_char, c_int, c_void};
use std::pin::Pin;
use std::ptr;
use std::slice;

/*
 * Future
 */

pub struct Future {
    fut: *mut fdb::FDBFuture,
    waker: AtomicWaker,
    registered: bool,
}

impl Future {
    pub fn new(fut: *mut fdb::FDBFuture) -> Self {
        Self {
            fut,
            waker: AtomicWaker::new(),
            registered: false,
        }
    }
}

impl futures::Future for Future {
    type Output = Result<ReadyFuture, Error>;

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> futures::Poll<Self::Output> {
        debug_assert!(!self.fut.is_null());

        if !self.registered {
            self.waker.register(waker);

            unsafe {
                fdb::fdb_future_set_callback(
                    self.fut,
                    Some(fdb_future_callback),
                    &self.waker as *const _ as *mut _,
                );
            }

            self.registered = true;
            return futures::Poll::Pending;
        }

        let ready = unsafe { fdb::fdb_future_is_ready(self.fut) };
        if ready == 0 {
            return futures::Poll::Pending;
        }

        let err = unsafe { fdb::fdb_future_get_error(self.fut) };
        if err != 0 {
            return futures::Poll::Ready(Err(Error { err }));
        }

        let res = ReadyFuture::new(replace(&mut self.fut, ptr::null_mut()));
        futures::Poll::Ready(Ok(res))
    }
}

impl Drop for Future {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}

extern "C" fn fdb_future_callback(
    _fut: *mut fdb::FDBFuture,
    callback_parameter: *mut c_void,
) {
    let awaker: *const AtomicWaker = callback_parameter as *const _;
    unsafe { (*awaker).wake() };
}

/*
 * Key
 */

pub struct Key {
    fut: *mut fdb::FDBFuture,
    key: *const u8,
    key_len: c_int,
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
    fut: *mut fdb::FDBFuture,
    val: *const u8,
    val_len: c_int,
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
    kv: *const fdb::FDBKeyValue,
    _phantom: PhantomData<&'a fdb::FDBKeyValue>,
}

impl<'a> KeyValue<'a> {
    pub fn key(&self) -> &[u8] {
        let kv = self.kv as *const RawKeyValue;
        unsafe { slice::from_raw_parts((*kv).key, (*kv).key_len as usize) }
    }

    pub fn value(&self) -> &[u8] {
        let kv = self.kv as *const RawKeyValue;
        unsafe { slice::from_raw_parts((*kv).value, (*kv).value_len as usize) }
    }
}

/*
 * KeyValueArray
 */

pub struct KeyValueArray {
    fut: *mut fdb::FDBFuture,
    kv: *const fdb::FDBKeyValue,
    count: c_int,
    more: fdb::fdb_bool_t,
}

impl KeyValueArray {
    pub fn get(&self, index: usize) -> KeyValue {
        KeyValue {
            kv: unsafe { self.kv.add(index) },
            _phantom: PhantomData,
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
    fut: *mut fdb::FDBFuture,
    strings: *mut *const c_char,
    count: c_int,
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

/*
 * ReadyFuture
 */

pub struct ReadyFuture {
    fut: *mut fdb::FDBFuture,
}

impl ReadyFuture {
    pub fn new(fut: *mut fdb::FDBFuture) -> Self {
        Self { fut }
    }

    pub fn into_cluster(self) -> Result<*mut fdb::FDBCluster, Error> {
        let mut val = ptr::null_mut();
        bail!(unsafe { fdb::fdb_future_get_cluster(self.fut, &mut val) });
        Ok(val)
    }

    pub fn into_database(self) -> Result<*mut fdb::FDBDatabase, Error> {
        let mut val = ptr::null_mut();
        bail!(unsafe { fdb::fdb_future_get_database(self.fut, &mut val) });
        Ok(val)
    }

    pub fn into_key(mut self) -> Result<Key, Error> {
        let mut key = ptr::null();
        let mut key_len = 0;
        bail!(unsafe { fdb::fdb_future_get_key(self.fut, &mut key, &mut key_len) });
        Ok(Key {
            fut: replace(&mut self.fut, ptr::null_mut()),
            key,
            key_len,
        })
    }

    pub fn into_value(mut self) -> Result<Option<Value>, Error> {
        let mut present = 0;
        let mut val = ptr::null();
        let mut val_len = 0;
        bail!(unsafe { fdb::fdb_future_get_value(self.fut, &mut present, &mut val, &mut val_len) });
        if present != 0 {
            Ok(Some(Value {
                fut: replace(&mut self.fut, ptr::null_mut()),
                val,
                val_len,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn into_keyvalue_array(mut self) -> Result<KeyValueArray, Error> {
        let mut kv = ptr::null();
        let mut count = 0;
        let mut more = 0;
        bail!(unsafe { fdb::fdb_future_get_keyvalue_array(self.fut, &mut kv, &mut count, &mut more) });
        Ok(KeyValueArray {
            fut: replace(&mut self.fut, ptr::null_mut()),
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
        let mut strings = ptr::null_mut();
        let mut count = 0;
        bail!(unsafe { fdb::fdb_future_get_string_array(self.fut, &mut strings, &mut count) });
        Ok(StringArray {
            fut: replace(&mut self.fut, ptr::null_mut()),
            strings,
            count,
        })
    }
}

impl Drop for ReadyFuture {
    fn drop(&mut self) {
        if !self.fut.is_null() {
            unsafe { fdb::fdb_future_destroy(self.fut) };
        }
    }
}