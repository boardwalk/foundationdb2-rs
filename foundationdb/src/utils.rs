use crate::transaction::{CommittedTransaction, FailedTransaction, Transaction};

pub async fn retry(f: impl Fn() -> Transaction) -> Result<CommittedTransaction, FailedTransaction> {
    loop {
        let tran = f();
        match await!(tran.commit()) {
            Ok(tran) => return Ok(tran),
            Err(tran) => await!(tran.on_error())?,
        }
    }
}
