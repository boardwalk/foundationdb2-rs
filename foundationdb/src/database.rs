use crate::error::Error;
use crate::options::DatabaseOption;
use crate::transaction::Transaction;
use foundationdb_sys as fdb;
use std::os::raw::c_int;
use std::ptr;

pub struct Database {
    pub(crate) database: *mut fdb::FDBDatabase,
}

impl Database {
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
        let mut tran = ptr::null_mut();
        bail!(unsafe { fdb::fdb_database_create_transaction(self.database, &mut tran) });
        Ok(Transaction { tran })
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe { fdb::fdb_database_destroy(self.database) };
    }
}
