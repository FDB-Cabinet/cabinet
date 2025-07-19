use crate::stats_workload::errors::StatsError;
use crate::stats_workload::wal::{StatsHolder, Wal};
use cabinet::with_cabinet;
use foundationdb::FdbBindingError;
use foundationdb_simulation::{Database, Metric, Metrics, RustWorkload, Severity, WorkloadContext};
use rand_chacha::rand_core::SeedableRng;

mod errors;
mod wal;

pub const STATS_WORKLOAD_NAME: &str = "StatsWorkload";

pub struct StatsWorkload {
    workload_context: WorkloadContext,
    iterations: usize,
    successful_iteration: usize,
    failed_iterations: usize,
    wal: Wal,
    stats_holder: StatsHolder,
}

impl StatsWorkload {
    pub fn new(workload_context: WorkloadContext, iterations: usize) -> Self {
        let seed =
            workload_context.shared_random_number() as u64 + workload_context.client_id() as u64;
        let rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);

        let wal = Wal::new(rng);

        Self {
            workload_context,
            iterations,
            successful_iteration: 0,
            failed_iterations: 0,
            wal,
            stats_holder: Default::default(),
        }
    }
}

impl RustWorkload for StatsWorkload {
    async fn setup(&mut self, db: Database) {
        if self.workload_context.client_id() != 0 {
            return;
        }

        loop {
            match self.init(&db).await {
                Ok(_) => {
                    self.workload_context.trace(
                        Severity::Info,
                        "StatsWorkload initialized successfully",
                        &[("Layer", "Rust"), ("Phase", "Setup")],
                    );

                    break;
                }
                Err(FdbBindingError::NonRetryableFdbError(_)) => {
                    self.workload_context.trace(
                        Severity::Warn,
                        "StatsWorkload initialization failed",
                        &[("Layer", "Rust"), ("Phase", "Setup")],
                    );

                    continue;
                }
                Err(_) => {
                    self.workload_context.trace(
                        Severity::Error,
                        "StatsWorkload initialization failed on retryable error. Retrying...",
                        &[("Layer", "Rust"), ("Phase", "Setup")],
                    );

                    break;
                }
            }
        }
    }

    async fn start(&mut self, db: Database) {
        let tenant = format!("tenant{}", self.workload_context.client_id());
        for iteration in 0..self.iterations {
            match self.simulate(&db, &tenant).await {
                Ok(_) => {
                    self.workload_context.trace(
                        Severity::Info,
                        "StatsWorkload simulation successfully",
                        &[
                            ("Layer", "Rust"),
                            ("Phase", "Start"),
                            ("Iteration", &format!("{}", iteration)),
                        ],
                    );
                    self.successful_iteration += 1;
                }
                Err(FdbBindingError::NonRetryableFdbError(_)) => {
                    self.workload_context.trace(
                        Severity::Warn,
                        "StatsWorkload simulation failed",
                        &[
                            ("Layer", "Rust"),
                            ("Phase", "Start"),
                            ("Iteration", &format!("{}", iteration)),
                        ],
                    );
                    self.failed_iterations += 1;
                }
                Err(err) => {
                    self.workload_context.trace(
                        Severity::Error,
                        "StatsWorkload simulation failed on retryable error. Retrying...",
                        &[
                            ("Layer", "Rust"),
                            ("Phase", "Start"),
                            ("Iteration", &format!("{}", iteration)),
                            ("Error", &err.to_string()),
                        ],
                    );
                    self.failed_iterations += 1;
                }
            }
        }
    }

    async fn check(&mut self, db: Database) {
        let tenant = format!("tenant{}", self.workload_context.client_id());

        loop {
            match self.verify(&db, &tenant).await {
                Ok(_) => {
                    println!("ok");
                    self.workload_context.trace(
                        Severity::Info,
                        "StatsWorkload verification successfully",
                        &[("Layer", "Rust"), ("Phase", "Check")],
                    );

                    break;
                }
                Err(FdbBindingError::NonRetryableFdbError(_)) => {
                    println!("non retryable error");
                    self.workload_context.trace(
                        Severity::Warn,
                        "StatsWorkload verification failed",
                        &[("Layer", "Rust"), ("Phase", "Check")],
                    );

                    continue;
                }
                Err(err) => {
                    let error = err.to_string();
                    self.workload_context.trace(
                        Severity::Error,
                        "StatsWorkload verification failed on retryable error.",
                        &[("Layer", "Rust"), ("Phase", "Check"), ("Error", &error)],
                    );

                    break;
                }
            }
        }
    }

    fn get_metrics(&self, mut out: Metrics) {
        out.push(Metric::val(
            "failed_iterations",
            self.failed_iterations as f64,
        ));
        out.push(Metric::val(
            "successful_iteration",
            self.successful_iteration as f64,
        ));
    }

    fn get_check_timeout(&self) -> f64 {
        5000.0
    }
}

impl StatsWorkload {
    async fn init(&mut self, _db: &Database) -> Result<(), FdbBindingError> {
        Ok(())
    }

    async fn verify(&mut self, db: &Database, tenant: &str) -> Result<(), FdbBindingError> {
        let expected_count = self.stats_holder.get_count() as i64;
        let expected_size = self.stats_holder.get_size() as i64;

        with_cabinet(&db, &tenant, |cabinet| async move {
            let stats = cabinet.get_stats();

            let actual_count = stats.get_count().await?;
            let actual_size = stats.get_size().await?;

            if stats.get_size().await? != expected_size {
                return Err(StatsError::InvalidDatabaseStatsSize {
                    actual: actual_size,
                    expected: expected_size,
                }
                .into());
            }

            if stats.get_count().await? != expected_count {
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

    async fn simulate(&mut self, db: &Database, tenant: &str) -> Result<(), FdbBindingError> {
        let event = self.wal.next_event(tenant);

        println!("{tenant} => {:?}", event);

        let result = with_cabinet(&db, tenant, |cabinet| async move {
            Ok(event.apply(cabinet).await?)
        })
        .await?;

        result.update_stats(&mut self.stats_holder);

        Ok(())
    }
}
