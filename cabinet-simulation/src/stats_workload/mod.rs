/// This module implements a workload for testing cabinet statistics functionality.
use crate::stats_workload::errors::StatsError;
use crate::stats_workload::wal::{StatsHolder, Wal};
use crate::workload::WorkloadLogic;
use rand::{rng, Rng};
use rand_chacha::rand_core::SeedableRng;
use toolbox::foundationdb::Database;
use toolbox::foundationdb::FdbBindingError;
use toolbox::foundationdb_simulation::WorkloadContext;
use toolbox::with_tenant;


mod errors;
mod wal;

/// Name of the statistics workload
pub const STATS_WORKLOAD_NAME: &str = "StatsWorkload";

/// Statistics workload structure that maintains WAL and statistics holder
pub struct StatsWorkload {
    /// Write-ahead log for tracking operations
    wal: Wal,
    /// Holder for maintaining statistics
    stats_holder: StatsHolder,
}

impl StatsWorkload {
    /// Creates a new StatsWorkload instance
    ///
    /// # Arguments
    /// * `workload_context` - Context containing workload execution parameters
    pub fn new(workload_context: &WorkloadContext) -> Self {
        let seed =
            workload_context.shared_random_number() as u64 + workload_context.client_id() as u64;
        let rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);

        let wal = Wal::new(rng);

        Self {
            wal,
            stats_holder: Default::default(),
        }
    }

    /// Gets the tenant name for the given context
    ///
    /// # Arguments
    /// * `ctx` - Workload context containing client ID
    fn get_tenant(&self, ctx: &WorkloadContext) -> String {
        format!("tenant{}", ctx.client_id())
    }
}

impl WorkloadLogic for StatsWorkload {
    /// Initializes the workload
    ///
    /// # Arguments
    /// * `_db` - Database instance
    /// * `_ctx` - Workload context
    async fn init(
        &mut self,
        _db: &Database,
        _ctx: &WorkloadContext,
    ) -> Result<(), FdbBindingError> {
        Ok(())
    }

    /// Verifies the workload statistics against database
    ///
    /// # Arguments
    /// * `db` - Database instance
    /// * `ctx` - Workload context
    async fn verify(
        &mut self,
        db: &Database,
        ctx: &WorkloadContext,
    ) -> Result<(), FdbBindingError> {
        let expected_count = self.stats_holder.get_count() as i64;
        let expected_size = self.stats_holder.get_size() as i64;
        let tenant = self.get_tenant(ctx);

        println!("Check for tenant {tenant}");

        with_tenant(db, &tenant, |cabinet| async move {
            let stats = cabinet.get_stats();

            let mut actual_count = stats.get_count().await?;
            let actual_size = stats.get_size().await?;

            let mut rng = rng();
            if rng.random_bool(0.01) {
                actual_count = 1;
            }

            if actual_size != expected_size {
                return Err(StatsError::InvalidDatabaseStatsSize {
                    actual: actual_size,
                    expected: expected_size,
                }
                .into());
            }

            if actual_count != expected_count {
                return Err(StatsError::InvalidDatabaseStatsCount {
                    actual: actual_count,
                    expected: expected_count,
                }
                .into());
            }

            Ok(())
        })
        .await?;

        Ok(())
    }

    /// Simulates workload operations
    ///
    /// # Arguments
    /// * `db` - Database instance  
    /// * `ctx` - Workload context
    async fn simulate(
        &mut self,
        db: &Database,
        ctx: &WorkloadContext,
    ) -> Result<(), FdbBindingError> {
        let tenant = self.get_tenant(ctx);
        let event = self.wal.next_event(&tenant);

        println!("{tenant} => {:?}", event);

        let result = with_tenant(&db, &tenant, |cabinet| async move {
            let result = event.apply(cabinet).await;
            if let Err(err) = &result {
                println!("***************------*****{err}");
            }
            Ok(result?)
        })
        .await;

        if let Err(err) = &result {
            println!("/////////////////{err:?}");
        }

        let result = result?;

        result.update_stats(&mut self.stats_holder);

        Ok(())
    }

    /// Returns the name of this workload
    fn name(&self) -> &'static str {
        STATS_WORKLOAD_NAME
    }
}
