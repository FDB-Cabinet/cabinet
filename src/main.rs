use cabinet::run;
use cabinet_lib::errors::CabinetLibError;
use toolbox::foundationdb::{Database, FdbBindingError};
use toolbox::with_transaction;

async fn cleanup(database: &Database) -> Result<(), FdbBindingError> {
    with_transaction(database, |trx| async move {
        trx.clear_range(b"\0", b"\xff");
        Ok(())
    })
    .await
}

#[tokio::main]
async fn main() -> Result<(), CabinetLibError> {
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
    }

    Ok(())
}
