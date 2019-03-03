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
mod outputs;
mod transaction;

// Everything is public except future::{Future, ReadyFuture}
pub use cluster::*;
pub use database::*;
pub use error::*;
pub use network::*;
pub use options::*;
pub use outputs::*;
pub use transaction::*;
