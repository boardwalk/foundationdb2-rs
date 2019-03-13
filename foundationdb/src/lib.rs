#![feature(futures_api)]
#![allow(clippy::new_ret_no_self)]

macro_rules! bail {
    ($e:expr) => {{
        let err = $e;
        if err != 0 {
            return Err(crate::error::Error { err });
        }
    }};
}

mod database;
mod error;
mod future;
#[cfg(feature = "async")]
mod future_async;
mod future_ready;
mod network;
mod options;
mod outputs;
mod transaction;
#[cfg(feature = "async")]
mod transaction_async;

// Everything is public except futures
pub use database::*;
pub use error::*;
pub use network::*;
pub use options::*;
pub use outputs::*;
pub use transaction::*;
