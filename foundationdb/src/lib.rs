#![feature(async_await, await_macro, futures_api)]
#![allow(clippy::new_ret_no_self)]

macro_rules! bail {
    ($e:expr) => {{
        let err = $e;
        if err != 0 {
            return Err(crate::error::Error { err });
        }
    }};
}

mod cluster;
mod database;
mod error;
mod future;
mod network;
mod options;
mod transaction;

pub use cluster::Cluster;
pub use database::Database;
pub use error::Error;
pub use future::{Key, KeyValue, KeyValueArray, StringArray, Value};
pub use network::Network;
pub use options::*;
pub use transaction::{GetRangeOpt, KeySelector, Transaction};

#[cfg(test)]
mod test {
    use super::*;
    use futures::executor::block_on;

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

        await!(database.transact(|tran| async {
            tran.set(b"hello", b"world");

            let value = await!(tran.get(b"hello", false))?;

            assert_eq!(value.as_ref().map(|v| v.as_ref()), Some(&b"world"[..]));

            tran.clear(b"hello");

            Ok(tran)
        }))?;

        Ok(())
    }
}
