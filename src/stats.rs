//! Stats module for managing statistics about items stored in the cabinet.
//! Tracks both item counts and total size of stored data.

use crate::errors::CabinetError;
use crate::item::Item;
use crate::prefix::{EntityType, Prefix, StatType};
use fdb_wrapper::foundationdb;
use foundationdb::options::MutationType;
use foundationdb::tuple::Subspace;
use foundationdb::RetryableTransaction;

/// Events that can trigger stat updates
pub enum StatEvent<'a> {
    /// Put a new item
    Put(&'a Item),
    /// Delete an existing item
    Delete(&'a Item),
    /// Delete all items
    DeleteAll,
}

/// Holds statistics and manages stat updates
pub struct StatsHolder {
    subspace: Subspace,
    transaction: RetryableTransaction,
}

impl StatsHolder {
    /// Creates a new StatsHolder
    ///
    /// # Parameters
    /// * `subspace` - Subspace to store stats in
    /// * `transaction` - Transaction to use for updates
    pub fn new(subspace: Subspace, transaction: RetryableTransaction) -> Self {
        Self {
            subspace: subspace.subspace(&Prefix::Stats),
            transaction,
        }
    }
}

/// Trait for different types of stats
#[async_trait::async_trait]
trait Stat {
    /// Updates stats after putting an item
    ///
    /// # Parameters
    /// * `item` - Item being put
    async fn update_put(&self, item: &Item) -> crate::errors::Result<()>;

    /// Updates stats after deleting an item
    ///
    /// # Parameters
    /// * `item` - Item being deleted
    async fn update_delete(&self, item: &Item) -> crate::errors::Result<()>;

    /// Updates stats after deleting all items
    async fn update_delete_all(&self) -> crate::errors::Result<()>;
}

impl StatsHolder {
    /// Gets the current item count
    ///
    /// # Returns
    /// Current number of items
    pub async fn get_count(&self) -> crate::errors::Result<i64> {
        let headcount_stats = HeadcountStats::new(self.subspace.clone(), &self.transaction);
        headcount_stats.get_count().await
    }

    /// Gets the current total size
    ///
    /// # Returns
    /// Total size of all items in bytes
    pub async fn get_size(&self) -> crate::errors::Result<i64> {
        let headcount_stats = SizeStats::new(self.subspace.clone(), &self.transaction);
        headcount_stats.get_size().await
    }

    /// Updates stats based on the provided event
    ///
    /// # Parameters
    /// * `stat_event` - Event triggering the update
    pub async fn update(&self, stat_event: StatEvent<'_>) -> crate::errors::Result<()> {
        let stat_holders: Vec<Box<dyn Stat>> = vec![
            Box::new(SizeStats::new(self.subspace.clone(), &self.transaction)),
            Box::new(HeadcountStats::new(
                self.subspace.clone(),
                &self.transaction,
            )),
        ];

        match stat_event {
            StatEvent::Put(item) => {
                for stat_holder in stat_holders.iter() {
                    stat_holder.update_put(item).await?;
                }
            }
            StatEvent::Delete(item) => {
                for stat_holder in stat_holders.iter() {
                    stat_holder.update_delete(item).await?;
                }
            }
            StatEvent::DeleteAll => {
                for stat_holder in stat_holders.iter() {
                    stat_holder.update_delete_all().await?;
                }
            }
        }

        Ok(())
    }
}

/// Tracks count of items
struct HeadcountStats<'a> {
    subspace: Subspace,
    transaction: &'a RetryableTransaction,
}

impl<'a> HeadcountStats<'a> {
    /// Creates a new HeadcountStats
    ///
    /// # Parameters
    /// * `subspace` - Subspace to store stats in
    /// * `transaction` - Transaction to use for updates
    pub fn new(subspace: Subspace, transaction: &'a RetryableTransaction) -> Self {
        Self {
            subspace: subspace.subspace(&EntityType::Headcount),
            transaction,
        }
    }

    /// Gets the current item count
    ///
    /// # Returns
    /// Current number of items
    pub async fn get_count(&self) -> crate::errors::Result<i64> {
        let stat_count_key = self.subspace.pack(&StatType::Value);
        let Some(value) = self.transaction.get(&stat_count_key, true).await? else {
            return Ok(0);
        };
        let value = i64::from_le_bytes(
            value
                .to_vec()
                .try_into()
                .map_err(|_| CabinetError::InvalidCountStatsValue)?,
        );
        Ok(value)
    }
}

#[async_trait::async_trait]
impl Stat for HeadcountStats<'_> {
    /// Updates stats after putting an item
    ///
    /// # Parameters
    /// * `_item` - Item being put
    async fn update_put(&self, _item: &Item) -> crate::errors::Result<()> {
        let stat_count_key = self.subspace.pack(&StatType::Value);

        let incremement = 1_i64.to_le_bytes();
        self.transaction
            .atomic_op(&stat_count_key, &incremement, MutationType::Add);

        Ok(())
    }

    /// Updates stats after deleting an item
    ///
    /// # Parameters
    /// * `_item` - Item being deleted
    async fn update_delete(&self, _item: &Item) -> crate::errors::Result<()> {
        let stat_count_key = self.subspace.pack(&StatType::Value);

        let incremement = (-1_i64).to_le_bytes();
        self.transaction
            .atomic_op(&stat_count_key, &incremement, MutationType::Add);
        Ok(())
    }

    /// Updates stats after deleting all items
    async fn update_delete_all(&self) -> crate::errors::Result<()> {
        let stat_count_key = self.subspace.pack(&StatType::Value);
        self.transaction.clear(&stat_count_key);
        Ok(())
    }
}

/// Tracks total size of items
struct SizeStats<'a> {
    subspace: Subspace,
    transaction: &'a RetryableTransaction,
}

impl<'a> SizeStats<'a> {
    /// Creates a new SizeStats
    ///
    /// # Parameters
    /// * `subspace` - Subspace to store stats in
    /// * `transaction` - Transaction to use for updates
    pub fn new(subspace: Subspace, transaction: &'a RetryableTransaction) -> Self {
        Self {
            subspace: subspace.subspace(&EntityType::Sizes),
            transaction,
        }
    }

    /// Gets the current total size
    ///
    /// # Returns
    /// Total size of all items in bytes
    pub async fn get_size(&self) -> crate::errors::Result<i64> {
        let stat_count_key = self.subspace.pack(&StatType::Value);
        let Some(value) = self.transaction.get(&stat_count_key, true).await? else {
            return Ok(0);
        };
        let value = i64::from_le_bytes(
            value
                .to_vec()
                .try_into()
                .map_err(|_| CabinetError::InvalidCountStatsValue)?,
        );
        Ok(value)
    }
}

#[async_trait::async_trait]
impl Stat for SizeStats<'_> {
    /// Updates stats after putting an item
    ///
    /// # Parameters
    /// * `item` - Item being put
    async fn update_put(&self, item: &Item) -> crate::errors::Result<()> {
        let stat_size_key = self.subspace.pack(&StatType::Value);
        let size = item.as_bytes().len() as i64;
        let size = size.to_le_bytes();
        self.transaction
            .atomic_op(&stat_size_key, &size, MutationType::Add);
        Ok(())
    }

    /// Updates stats after deleting an item
    ///
    /// # Parameters
    /// * `item` - Item being deleted
    async fn update_delete(&self, item: &Item) -> crate::errors::Result<()> {
        let stat_size_key = self.subspace.pack(&StatType::Value);
        let size = item.as_bytes().len() as i64;
        let size = (-size).to_le_bytes();
        self.transaction
            .atomic_op(&stat_size_key, &size, MutationType::Add);
        Ok(())
    }

    /// Updates stats after deleting all items
    async fn update_delete_all(&self) -> crate::errors::Result<()> {
        let stat_size_key = self.subspace.pack(&StatType::Value);
        self.transaction.clear(&stat_size_key);
        Ok(())
    }
}
