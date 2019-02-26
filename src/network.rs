use foundationdb_sys as fdb;
use std::thread::{self, JoinHandle};
use crate::future::Error;
use std::os::raw::c_int;

/*
 * Missing:
 * fdb_network_set_option
 */

pub struct Network {
    join_handle: Option<JoinHandle<()>>,
}

impl Network {
    pub fn new() -> Result<Self, Error> {
        bail!(unsafe { fdb::fdb_select_api_version_impl(fdb::FDB_API_VERSION as c_int, fdb::FDB_API_VERSION as c_int) });
        bail!(unsafe { fdb::fdb_setup_network() });

        let join_handle = thread::spawn(|| Network::run().unwrap());

        Ok(Network {
            join_handle: Some(join_handle),
        })
    }

    fn run() -> Result<(), Error> {
        bail!(unsafe { fdb::fdb_run_network() });

        Ok(())
    }

    fn stop() -> Result<(), Error> {
        bail!(unsafe { fdb::fdb_stop_network() });

        Ok(())
    }
}

impl Drop for Network {
    fn drop(&mut self) {
        Network::stop().unwrap();

        let join_handle = self.join_handle.take().unwrap();
        join_handle.join().unwrap();
    }
}
