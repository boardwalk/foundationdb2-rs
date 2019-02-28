use crate::error::Error;
use foundationdb_sys as fdb;
use std::os::raw::c_int;
use std::thread::{self, JoinHandle};

/*
 * Missing:
 * fdb_network_set_option
 */

pub struct Network {
    join_handle: Option<JoinHandle<()>>,
}

impl Network {
    pub fn new() -> Result<Self, Error> {
        bail!(unsafe {
            fdb::fdb_select_api_version_impl(
                fdb::FDB_API_VERSION as c_int,
                fdb::FDB_API_VERSION as c_int,
            )
        });
        bail!(unsafe { fdb::fdb_setup_network() });

        let join_handle = thread::spawn(|| Network::run().unwrap());

        Ok(Network {
            join_handle: Some(join_handle),
        })
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if let Some(join_handle) = self.join_handle.take() {
            bail!(unsafe { fdb::fdb_stop_network() });
            let unknown_error = 4000;
            join_handle.join().map_err(|_| Error { err: unknown_error })?;
        }

        Ok(())
    }

    fn run() -> Result<(), Error> {
        bail!(unsafe { fdb::fdb_run_network() });

        Ok(())
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}
