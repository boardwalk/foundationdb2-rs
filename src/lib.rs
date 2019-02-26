#![feature(async_await, await_macro, futures_api)]
#![allow(clippy::new_ret_no_self)]

macro_rules! bail {
    ($e:expr) => {{
        let err = $e;
        if err != 0 {
            return Err(crate::future::Error::new(err));
        }
    }};
}

mod cluster;
mod database;
mod future;
mod network;
mod transaction;

pub use cluster::Cluster;
pub use database::Database;
pub use future::{Error, Value};
pub use network::Network;
pub use transaction::{retry, Transaction};

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
        let cluster = await!(Cluster::new("/Users/boardwalk/Code/foundationdb-build/fdb.cluster"))?;
        let _database = await!(cluster.create_database())?;
        Ok(())
    }
}