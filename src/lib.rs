use foundationdb::options::TransactionOption;
use foundationdb::{Database, FdbBindingError, RetryableTransaction};

pub mod cabinet;
pub mod errors;
pub mod item;
mod prefix;
mod stats;

pub async fn with_transaction<F, Fut, T>(database: &Database, f: F) -> Result<T, FdbBindingError>
where
    F: Fn(RetryableTransaction) -> Fut + Clone,
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
