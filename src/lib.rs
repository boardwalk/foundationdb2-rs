#![feature(async_await, await_macro, futures_api)]
#![allow(clippy::new_ret_no_self)]

macro_rules! bail {
    ($e:expr) => {
        {
            let err = $e;
            if err != 0 {
                return Err(crate::future::Error::new(err));
            }
        }
    };
}

mod cluster;
mod database;
mod future;
mod network;
mod transaction;

pub use cluster::Cluster;
pub use database::Database;
pub use network::Network;
pub use transaction::{retry, Transaction};
