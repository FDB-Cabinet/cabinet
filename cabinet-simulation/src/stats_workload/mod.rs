use crate::stats_workload::errors::StatsError;
use cabinet::cabinet::Cabinet;
use cabinet::item::Item;
use cabinet::with_transaction;
use foundationdb::FdbBindingError;
use foundationdb_simulation::{Database, Metric, Metrics, RustWorkload, Severity, WorkloadContext};

mod errors;

pub const STATS_WORKLOAD_NAME: &str = "StatsWorkload";

pub struct StatsWorkload {
    workload_context: WorkloadContext,
    iterations: usize,
    successful_iteration: usize,
    failed_iterations: usize,
}

impl StatsWorkload {
    pub fn new(workload_context: WorkloadContext, iterations: usize) -> Self {
        Self {
            workload_context,
            iterations,
            successful_iteration: 0,
            failed_iterations: 0,
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
        for iteration in 0..self.iterations {
            match self.simulate(&db).await {
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
                Err(_) => {
                    self.workload_context.trace(
                        Severity::Error,
                        "StatsWorkload simulation failed on retryable error. Retrying...",
                        &[
                            ("Layer", "Rust"),
                            ("Phase", "Start"),
                            ("Iteration", &format!("{}", iteration)),
                        ],
                    );
                    self.failed_iterations += 1;
                }
            }
        }
    }

    async fn check(&mut self, db: Database) {
        if self.workload_context.client_id() != 0 {
            return;
        }

        loop {
            match self.verify(&db).await {
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
    async fn init(&mut self, db: &Database) -> Result<(), FdbBindingError> {
        Ok(())
    }

    async fn verify(&mut self, db: &Database) -> Result<(), FdbBindingError> {
        with_transaction(&db, |tr| async move {
            let cabinet = Cabinet::new(&tr);

            let Some(item) = cabinet.get(b"key").await? else {
                return Err(StatsError::ItemNotFound.into());
            };

            if item.value != b"value2" {
                return Err(StatsError::ItemValueIncorrect {
                    expected: b"value2".to_vec(),
                    actual: item.value,
                }
                .into());
            }

            Ok(())
        })
        .await?;

        Ok(())
    }

    async fn simulate(&mut self, db: &Database) -> Result<(), FdbBindingError> {
        with_transaction(&db, |tr| async move {
            let cabinet = Cabinet::new(&tr);

            let item = Item::new(b"key", b"value");

            cabinet.put(&item).await?;

            Ok(())
        })
        .await?;

        Ok(())
    }
}
