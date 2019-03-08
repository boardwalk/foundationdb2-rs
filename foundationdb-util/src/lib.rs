#![feature(async_await, await_macro, futures_api)]

mod subspace;
mod tuple;

pub use subspace::Subspace;

use foundationdb::{CommittedTransaction, Database, Error, Transaction};
use futures::Future;

// TODO Fun should take &Transaction, but I can't get the lifetimes right
pub async fn transact<Fun, Fut>(db: &Database, f: Fun) -> Result<CommittedTransaction, Error>
where
    Fun: Fn(Transaction) -> Fut + 'static,
    Fut: Future<Output = Result<Transaction, Error>>,
{
    let mut tran = db.create_transaction()?;
    loop {
        tran = await!(f(tran))?;
        match await!(tran.commit()) {
            Ok(t) => return Ok(t),
            Err(t) => match await!(t.on_error()) {
                Ok(t) => tran = t,
                Err(t) => return Err(t.into_error()),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::transact;
    use foundationdb::{Database, Error, Network};
    use futures::executor::block_on;

    #[test]
    fn test_general() {
        let _network = Network::new().unwrap();
        block_on(test_general_async()).unwrap();
    }

    async fn test_general_async() -> Result<(), Error> {
        let database = Database::new()?;

        await!(transact(&database, |tran| async {
            tran.set(b"hello", b"world");
            let value = await!(tran.get(b"hello", false))?;
            assert_eq!(value.as_ref().map(|v| v.as_ref()), Some(&b"world"[..]));
            tran.clear(b"hello");
            Ok(tran)
        }))?;

        Ok(())
    }
}
