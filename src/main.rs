use cabinet::errors::CabinetError;
use cabinet::item::Item;
use cabinet::with_cabinet;
use ::cabinet::with_transaction;
use fdb_wrapper::foundationdb;
use fdb_wrapper::foundationdb::{Database, FdbBindingError};

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

    let database = Database::new_compat(None)
        .await
        .expect("Failed to create database");
    cleanup(&database).await?;

    let tenant = "tenant";

    with_cabinet(&database, tenant, |cabinet| async move {
        let item = Item::new(b"key", b"value");

        cabinet.put(&item).await?;

        let item = Item::new(b"key2", b"value2");

        cabinet.put(&item).await?;

        Ok(())
    })
    .await?;

    let count = with_cabinet(&database, tenant, |cabinet| async move {
        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("{count}");

    let item = with_cabinet(&database, tenant, |cabinet| async move {
        let item = cabinet.get(b"key").await?;

        Ok(item)
    })
    .await?;

    with_cabinet(&database, tenant, |cabinet| async move {
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

    let count = with_cabinet(&database, tenant, |cabinet| async move {
        cabinet.delete(b"key").await?;

        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("count: {count}");

    let count = with_cabinet(&database, tenant, |cabinet| async move {
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
