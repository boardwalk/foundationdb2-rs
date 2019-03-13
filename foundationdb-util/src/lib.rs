mod subspace;
mod tuple;

pub use subspace::Subspace;

use foundationdb::{Database, Error, Transaction};

pub fn transact<Fun, OutVal, OutErr>(db: &Database, mut f: Fun) -> Result<OutVal, OutErr>
where
    Fun: FnMut(&Transaction) -> Result<OutVal, OutErr>,
    OutErr: From<Error>,
{
    let mut tran = db.create_transaction()?;
    loop {
        let val = f(&tran)?;
        match tran.commit() {
            Ok(_) => return Ok(val),
            Err(fail_tran) => match fail_tran.on_error() {
                Ok(reset_tran) => tran = reset_tran,
                Err(fail_tran) => return Err(OutErr::from(fail_tran.into_error())),
            },
        }
    }
}

// TODO: Implement transact_async for async API

#[cfg(test)]
mod test {
    use crate::transact;
    use foundationdb::{Database, Error, Network};

    #[test]
    fn test_general() {
        let _network = Network::new().unwrap();
        let database = Database::new().unwrap();

        let _: Result<(), Error> = transact(&database, |tran| {
            tran.set(b"hello", b"world");
            let value = tran.get(b"hello", false)?;
            assert_eq!(value.as_ref().map(|v| v.as_ref()), Some(&b"world"[..]));
            tran.clear(b"hello");
            Ok(())
        });
    }
}
