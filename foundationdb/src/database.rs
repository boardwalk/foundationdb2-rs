use crate::error::Error;
use crate::transaction::{CommittedTransaction, Transaction};
use foundationdb_sys as fdb;
use futures::Future;
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

    // TODO Fun should take &Transaction, but I can't get the lifetimes right
    pub async fn transact<Fun, Fut>(&self, f: Fun) -> Result<CommittedTransaction, Error>
    where
        Fun: Fn(Transaction) -> Fut + 'static,
        Fut: Future<Output = Result<Transaction, Error>>,
    {
        let mut tran = self.create_transaction()?;
        loop {
            tran = await!(f(tran))?;
            match await!(tran.commit()) {
                Ok(t) => return Ok(t),
                Err(t) => {
                    match await!(t.on_error()) {
                        Ok(t) => tran = t,
                        Err(t) => return Err(t.into_error()),
                    }
                }
            }
        }
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe { fdb::fdb_database_destroy(self.database) };
    }
}
