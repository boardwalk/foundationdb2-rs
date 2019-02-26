use foundationdb_sys as fdb;
use crate::future::Error;
use crate::transaction::Transaction;
use std::ptr;

pub struct Database {
    pub(crate) database: *mut fdb::FDBDatabase,
}

impl Database {
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
