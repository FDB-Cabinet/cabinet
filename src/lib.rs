use crate::cabinet::Cabinet;
use foundationdb::options::TransactionOption;
use foundationdb::{Database, FdbBindingError, RetryableTransaction};

pub mod cabinet;
pub mod errors;
pub mod item;
mod prefix;
mod stats;

pub async fn with_transaction<'a, F, Fut, T>(
    database: &Database,
    f: F,
) -> Result<T, FdbBindingError>
where
    F: FnOnce(RetryableTransaction) -> Fut + Clone,
    Fut: Future<Output = Result<T, FdbBindingError>>,
{
    database
        .run(|trx, _| {
            let f = f.clone();

            async move {
                trx.set_option(TransactionOption::AutomaticIdempotency)?;
                f(trx).await
            }
        })
        .await
}

pub async fn with_cabinet<'a, F, Fut, T>(
    database: &Database,
    tenant: &str,
    f: F,
) -> Result<T, FdbBindingError>
where
    F: FnOnce(Cabinet) -> Fut + Clone,
    Fut: Future<Output = Result<T, FdbBindingError>>,
{
    with_transaction(database, |trx| {
        let f = f.clone();

        async move {
            let cabinet = Cabinet::new(trx, tenant);
            f(cabinet).await
        }
    })
    .await
}
