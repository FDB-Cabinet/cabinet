//! Module for managing Cabinet storage functionality
//! Provides interface to store, retrieve and manage items in FoundationDB

use crate::item::Item;
use crate::prefix::Prefix;
use crate::stats::{StatEvent, StatsHolder};
use foundationdb::tuple::Subspace;
use foundationdb::RetryableTransaction;

/// Cabinet provides item storage functionality with tenant isolation
pub struct Cabinet {
    /// The foundationdb transaction
    transaction: RetryableTransaction,
    /// Subspace for tenant isolation
    root_subspace: Subspace,
    /// Stats holder for tracking operations
    stats: StatsHolder,
}

impl Cabinet {
    /// Creates a new Cabinet instance for a tenant
    ///
    /// # Parameters
    /// * `transaction` - The foundationdb transaction
    /// * `tenant` - Tenant identifier for isolation
    ///
    /// # Returns
    /// New Cabinet instance
    pub fn new(transaction: RetryableTransaction, tenant: &str) -> Self {
        let root_subspace = Subspace::all().subspace(&tenant);
        let stats = StatsHolder::new(root_subspace.clone(), transaction.clone());
        Self {
            transaction,
            root_subspace,
            stats,
        }
    }
}

impl Cabinet {
    /// Stores an item in the cabinet
    ///
    /// # Parameters
    /// * `item` - The item to store
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn put(&self, item: &Item) -> crate::errors::Result<()> {
        let key = item.get_key();
        let data = item.as_bytes();
        let item_key = self.root_subspace.subspace(&Prefix::Data).pack(&key);
        self.transaction.set(&item_key, &data);
        self.stats.update(StatEvent::Put(&item)).await?;

        Ok(())
    }

    /// Retrieves an item by key
    ///
    /// # Parameters
    /// * `key` - Key of the item to retrieve
    ///
    /// # Returns
    /// Result containing Option<Item> if found
    pub async fn get(&self, key: &[u8]) -> crate::errors::Result<Option<Item>> {
        let item_key = self.root_subspace.subspace(&Prefix::Data).pack(&key);
        let Some(raw) = self.transaction.get(&item_key, true).await? else {
            return Ok(None);
        };
        let item = Item::from_bytes(&raw);
        Ok(Some(item))
    }

    /// Deletes an item by key
    ///
    /// # Parameters
    /// * `key` - Key of the item to delete
    ///
    /// # Returns
    /// Result containing Option<Item> if item was found and deleted
    pub async fn delete(&self, key: &[u8]) -> crate::errors::Result<Option<Item>> {
        let item_key = self.root_subspace.subspace(&Prefix::Data).pack(&key);

        let Some(item) = self.get(&key).await? else {
            return Ok(None);
        };

        self.transaction.clear(&item_key);
        self.stats.update(StatEvent::Delete(&item)).await?;

        Ok(Some(item))
    }

    /// Clears all items in the cabinet
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn clear(&self) -> crate::errors::Result<()> {
        let prefix = self.root_subspace.subspace(&Prefix::Data);
        let (start, end) = prefix.range();
        self.transaction.clear_range(&start, &end);
        self.stats.update(StatEvent::DeleteAll).await?;

        Ok(())
    }

    /// Gets the stats holder
    ///
    /// # Returns
    /// Reference to the StatsHolder
    pub fn get_stats(&self) -> &StatsHolder {
        &self.stats
    }
}
