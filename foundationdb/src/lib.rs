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
pub use transaction::{CommittedTransaction, FailedTransaction, GetRangeOpt, KeySelector, Transaction};
