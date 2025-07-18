use crate::errors::CabinetError;
use crate::item::Item;
use crate::prefix::Prefix;
use crate::stats::{StatEvent, StatsHolder};
use foundationdb::tuple::Subspace;
use foundationdb::RetryableTransaction;

pub struct Cabinet<'a> {
    transaction: &'a RetryableTransaction,
    root_subspace: Subspace,
    stats: StatsHolder<'a>,
}

impl<'a> Cabinet<'a> {
    pub fn new(transaction: &'a RetryableTransaction) -> Self {
        let root_subspace = Subspace::all();
        let stats = StatsHolder::new(root_subspace.clone(), transaction);
        Self {
            transaction,
            root_subspace,
            stats,
        }
    }
}

impl Cabinet<'_> {
    pub async fn put(&self, item: &Item) -> crate::errors::Result<()> {
        let key = item.get_key();
        let data = item.as_bytes();
        let item_key = self.root_subspace.subspace(&Prefix::Data).pack(&key);
        self.transaction.set(&item_key, &data);
        self.stats.update(StatEvent::Put(&item)).await?;

        Ok(())
    }

    pub async fn get(&self, key: &[u8]) -> crate::errors::Result<Option<Item>> {
        let item_key = self.root_subspace.subspace(&Prefix::Data).pack(&key);
        let Some(raw) = self.transaction.get(&item_key, true).await? else {
            return Ok(None);
        };
        let item = Item::from_bytes(&raw);
        Ok(Some(item))
    }

    pub async fn delete(&self, key: &[u8]) -> crate::errors::Result<()> {
        let item_key = self.root_subspace.subspace(&Prefix::Data).pack(&key);

        let Some(item) = self.get(&key).await? else {
            return Err(CabinetError::ItemNotFound(
                String::from_utf8_lossy(&key).to_string(),
            ));
        };

        self.transaction.clear(&item_key);
        self.stats.update(StatEvent::Delete(&item)).await?;

        Ok(())
    }

    pub async fn clear(&self) -> crate::errors::Result<()> {
        let prefix = self.root_subspace.subspace(&Prefix::Data);
        let (start, end) = prefix.range();
        self.transaction.clear_range(&start, &end);
        self.stats.update(StatEvent::DeleteAll).await?;

        Ok(())
    }

    pub fn get_stats(&self) -> &StatsHolder<'_> {
        &self.stats
    }
}
