use cabinet::errors::CabinetError;
use cabinet::item::Item;
use toolbox::foundationdb::{Database, FdbBindingError};
use toolbox::{with_tenant, with_transaction};

async fn cleanup(database: &Database) -> Result<(), FdbBindingError> {
    with_transaction(database, |trx| async move {
        trx.clear_range(b"\0", b"\xff");
        Ok(())
    })
    .await
}

#[tokio::main]
async fn main() -> Result<(), CabinetError> {
    let _guard = toolbox::get_network_thread()?;

    let database = Database::new_compat(None)
        .await
        .expect("Failed to create database");
    cleanup(&database).await?;

    let tenant = "tenant";

    with_tenant(&database, tenant, |cabinet| async move {
        let item = Item::new(b"key", b"value");

        cabinet.put(&item).await?;

        let item = Item::new(b"key2", b"value2");

        cabinet.put(&item).await?;

        Ok(())
    })
    .await?;

    let count = with_tenant(&database, tenant, |cabinet| async move {
        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("{count}");

    let item = with_tenant(&database, tenant, |cabinet| async move {
        let item = cabinet.get::<Item>(b"key").await?;

        Ok(item)
    })
    .await?;

    with_tenant(&database, tenant, |cabinet| async move {
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

    let count = with_tenant(&database, tenant, |cabinet| async move {
        cabinet.delete::<Item>(b"key").await?;

        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("count: {count}");

    let count = with_tenant(&database, tenant, |cabinet| async move {
        let size = cabinet.get_stats().get_size().await?;
        println!("size: {size}");

        cabinet.clear::<Item>().await?;

        let count = cabinet.get_stats().get_count().await?;

        Ok(count)
    })
    .await?;

    println!("{count}");

    Ok(())
}
