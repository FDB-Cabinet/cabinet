use crate::cabinet::Cabinet;
use crate::errors::CabinetError;
use crate::item::Item;
use foundationdb::options::TransactionOption;
use foundationdb::{Database, FdbBindingError, RetryableTransaction};

pub mod cabinet;
pub mod errors;
mod item;
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

async fn cleanup(database: &Database) -> Result<(), FdbBindingError> {
    with_transaction(database, |trx| async move {
        trx.clear_range(b"\0", b"\xff");
        Ok(())
    })
    .await
}

#[tokio::main]
async fn main() -> Result<(), CabinetError> {
    let _guard = unsafe { foundationdb::boot() };

    let database = Database::new(None).expect("Failed to create database");
    cleanup(&database).await?;

    with_transaction(&database, |trx| async move {
        let cabinet = Cabinet::new(&trx);

        let item = Item::new(b"key", b"value");

        cabinet.put(&item).await?;

        let item = Item::new(b"key2", b"value2");

        cabinet.put(&item).await?;

        Ok(())
    })
    .await?;

    let count = with_transaction(&database, |trx| async move {
        let cabinet = Cabinet::new(&trx);

        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("{count}");

    let item = with_transaction(&database, |trx| async move {
        let cabinet = Cabinet::new(&trx);

        let item = cabinet.get(b"key").await?;

        Ok(item)
    })
    .await?;

    with_transaction(&database, |trx| async move {
        let cabinet = Cabinet::new(&trx);

        for i in 0..1000 {
            let item = Item::new(
                format!("key{}", i).as_bytes(),
                format!("value{}", i).as_bytes(),
            );
            cabinet.put(&item).await?;
        }

        Ok(())
    })
    .await?;

    println!("{item:?}");

    let count = with_transaction(&database, |trx| async move {
        let cabinet = Cabinet::new(&trx);

        cabinet.delete(b"key").await?;

        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("count: {count}");

    let count = with_transaction(&database, |trx| async move {
        let cabinet = Cabinet::new(&trx);

        let size = cabinet.get_stats().get_size().await?;
        println!("size: {size}");

        cabinet.clear().await?;

        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("{count}");

    Ok(())
}
