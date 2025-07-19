use crate::errors::CabinetError;
use crate::item::Item;
use crate::prefix::{EntityType, Prefix, StatType};
use foundationdb::options::MutationType;
use foundationdb::tuple::Subspace;
use foundationdb::RetryableTransaction;

pub enum StatEvent<'a> {
    Put(&'a Item),
    Delete(&'a Item),
    DeleteAll,
}

pub struct StatsHolder {
    subspace: Subspace,
    transaction: RetryableTransaction,
}

impl StatsHolder {
    pub fn new(subspace: Subspace, transaction: RetryableTransaction) -> Self {
        Self {
            subspace: subspace.subspace(&Prefix::Stats),
            transaction,
        }
    }
}

#[async_trait::async_trait]
trait Stat {
    async fn update_put(&self, _item: &Item) -> crate::errors::Result<()>;
    async fn update_delete(&self, _item: &Item) -> crate::errors::Result<()>;
    async fn update_delete_all(&self) -> crate::errors::Result<()>;
}

impl StatsHolder {
    pub async fn get_count(&self) -> crate::errors::Result<i64> {
        let headcount_stats = HeadcountStats::new(self.subspace.clone(), &self.transaction);
        headcount_stats.get_count().await
    }

    pub async fn get_size(&self) -> crate::errors::Result<i64> {
        let headcount_stats = SizeStats::new(self.subspace.clone(), &self.transaction);
        headcount_stats.get_size().await
    }

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

struct HeadcountStats<'a> {
    subspace: Subspace,
    transaction: &'a RetryableTransaction,
}

impl<'a> HeadcountStats<'a> {
    pub fn new(subspace: Subspace, transaction: &'a RetryableTransaction) -> Self {
        Self {
            subspace: subspace.subspace(&EntityType::Headcount),
            transaction,
        }
    }

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
    async fn update_put(&self, _item: &Item) -> crate::errors::Result<()> {
        let stat_count_key = self.subspace.pack(&StatType::Value);

        let incremement = 1_i64.to_le_bytes();
        self.transaction
            .atomic_op(&stat_count_key, &incremement, MutationType::Add);

        Ok(())
    }

    async fn update_delete(&self, _item: &Item) -> crate::errors::Result<()> {
        let stat_count_key = self.subspace.pack(&StatType::Value);

        let incremement = (-1_i64).to_le_bytes();
        self.transaction
            .atomic_op(&stat_count_key, &incremement, MutationType::Add);
        Ok(())
    }

    async fn update_delete_all(&self) -> crate::errors::Result<()> {
        let stat_count_key = self.subspace.pack(&StatType::Value);
        self.transaction.clear(&stat_count_key);
        Ok(())
    }
}

struct SizeStats<'a> {
    subspace: Subspace,
    transaction: &'a RetryableTransaction,
}

impl<'a> SizeStats<'a> {
    pub fn new(subspace: Subspace, transaction: &'a RetryableTransaction) -> Self {
        Self {
            subspace: subspace.subspace(&EntityType::Sizes),
            transaction,
        }
    }

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
    async fn update_put(&self, item: &Item) -> crate::errors::Result<()> {
        let stat_size_key = self.subspace.pack(&StatType::Value);
        let size = item.as_bytes().len() as i64;
        let size = size.to_le_bytes();
        self.transaction
            .atomic_op(&stat_size_key, &size, MutationType::Add);
        Ok(())
    }

    async fn update_delete(&self, item: &Item) -> crate::errors::Result<()> {
        let stat_size_key = self.subspace.pack(&StatType::Value);
        let size = item.as_bytes().len() as i64;
        let size = (-size).to_le_bytes();
        self.transaction
            .atomic_op(&stat_size_key, &size, MutationType::Add);
        Ok(())
    }

    async fn update_delete_all(&self) -> crate::errors::Result<()> {
        let stat_size_key = self.subspace.pack(&StatType::Value);
        self.transaction.clear(&stat_size_key);
        Ok(())
    }
}
