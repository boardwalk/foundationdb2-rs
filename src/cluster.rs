use crate::database::Database;
use crate::future::{Error, Future};
use foundationdb_sys as fdb;
use std::ffi::CString;
use std::os::raw::c_int;

pub struct Cluster {
    cluster: *mut fdb::FDBCluster,
}

impl Cluster {
    pub async fn new(cluster_file_path: &str) -> Result<Self, Error> {
        let cluster_file_path = CString::new(cluster_file_path).unwrap();
        let fut = unsafe { fdb::fdb_create_cluster(cluster_file_path.as_ptr()) };
        let rfut = await!(Future::new(fut))?;
        let cluster = rfut.into_cluster()?;
        Ok(Self { cluster })
    }

    pub async fn create_database(&self) -> Result<Database, Error> {
        let db_name = "DB";
        let fut = unsafe {
            fdb::fdb_cluster_create_database(
                self.cluster,
                db_name.as_ptr(),
                db_name.len() as c_int,
            )
        };
        let rfut = await!(Future::new(fut))?;
        let database = rfut.into_database()?;
        Ok(Database { database })
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        unsafe { fdb::fdb_cluster_destroy(self.cluster) };
    }
}
