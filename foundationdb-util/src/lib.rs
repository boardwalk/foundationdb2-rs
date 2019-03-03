#![feature(async_await, await_macro, futures_api)]

use foundationdb::{Database, Transaction, CommittedTransaction, Error};
use futures::Future;

// TODO Fun should take &Transaction, but I can't get the lifetimes right
pub async fn transact<Fun, Fut>(db: &Database, f: Fun) -> Result<CommittedTransaction, Error>
where
    Fun: Fn(Transaction) -> Fut + 'static,
    Fut: Future<Output = Result<Transaction, Error>>
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
    use foundationdb::{Cluster, Error, Network};
    use futures::executor::block_on;
    use super::transact;

    #[test]
    fn test_general() {
        let _network = Network::new().unwrap();
        block_on(test_general_async()).unwrap();
    }

    async fn test_general_async() -> Result<(), Error> {
        // Note: If you create a cluster and a database and then immediately shut down,
        // the network thread will crash. I should create a C repro and post an issue.

        let cluster = await!(Cluster::new("/Users/boardwalk/Code/foundationdb-build/fdb.cluster"))?;
        let database = await!(cluster.create_database())?;

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
