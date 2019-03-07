use crate::error::Error;
use crate::options::DatabaseOption;
use crate::transaction::Transaction;
use foundationdb_sys as fdb;
use std::ffi::{CString, OsStr};
use std::os::{raw::c_int, unix::ffi::OsStrExt};
use std::ptr::null_mut;
use std::path::Path;

pub struct Database {
    pub(crate) database: *mut fdb::FDBDatabase,
}

impl Database {
    pub fn new() -> Result<Self, Error> {
        Self::from_cluster_file(Path::new(""))
    }

    // TODO: OsStrExt::as_bytes here is unix only, preventing compilation on Windows
    pub fn from_cluster_file(cluster_file: &Path) -> Result<Self, Error> {
        let cluster_file: &OsStr = cluster_file.as_ref();
        let cluster_file = CString::new(cluster_file.as_bytes()).unwrap();
        let mut database = null_mut();
        bail!(unsafe { fdb::fdb_create_database(cluster_file.as_ptr(), &mut database) });
        Ok(Self { database })
    }

    pub fn set_option(&self, option: DatabaseOption, value: &[u8]) -> Result<(), Error> {
        bail!(unsafe {
            fdb::fdb_database_set_option(
                self.database,
                option.as_c_enum(),
                value.as_ptr(),
                value.len() as c_int,
            )
        });

        Ok(())
    }

    pub fn create_transaction(&self) -> Result<Transaction, Error> {
        let mut tran = null_mut();
        bail!(unsafe { fdb::fdb_database_create_transaction(self.database, &mut tran) });
        Ok(Transaction { tran })
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe { fdb::fdb_database_destroy(self.database) };
    }
}
