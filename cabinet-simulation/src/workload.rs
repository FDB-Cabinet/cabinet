use foundationdb::FdbBindingError;
use foundationdb_simulation::{Database, Metric, Metrics, RustWorkload, Severity, WorkloadContext};

pub trait WorkloadLogic {
    /// Initialize the workload with the given database and context
    ///
    /// # Parameters
    /// * `db` - Reference to the database to initialize
    /// * `ctx` - Reference to the workload context containing configuration
    async fn init(&mut self, db: &Database, ctx: &WorkloadContext) -> Result<(), FdbBindingError>;

    /// Verify the workload results with the given database and context
    ///
    /// # Parameters
    /// * `db` - Reference to the database to verify
    /// * `ctx` - Reference to the workload context containing configuration
    async fn verify(&mut self, db: &Database, ctx: &WorkloadContext)
    -> Result<(), FdbBindingError>;

    /// Run a single simulation iteration with the given database and context
    ///
    /// # Parameters
    /// * `db` - Reference to the database to simulate on
    /// * `ctx` - Reference to the workload context containing configuration
    async fn simulate(
        &mut self,
        db: &Database,
        ctx: &WorkloadContext,
    ) -> Result<(), FdbBindingError>;

    /// Return the name of this workload
    fn name(&self) -> &'static str;

    /// Add any extra metrics for this workload to the given metrics collection
    ///
    /// # Parameters
    /// * `_out` - Mutable reference to metrics collection to add to
    fn extra_metrics(&self, _out: &mut Metrics) {}

    /// Optionally, override the default check timeout (in milliseconds)
    fn override_check_timeout(&self) -> Option<f64> {
        None
    }
}

pub struct Workload<W: WorkloadLogic> {
    workload_context: WorkloadContext,
    iterations: usize,
    successful_iteration: usize,
    failed_iterations: usize,
    workload_logic: W,
}

impl<W: WorkloadLogic> Workload<W> {
    pub fn new(workload_context: WorkloadContext, iterations: usize, workload_logic: W) -> Self {
        Self {
            workload_context,
            iterations,
            successful_iteration: 0,
            failed_iterations: 0,
            workload_logic,
        }
    }
}

impl<W: WorkloadLogic> RustWorkload for Workload<W> {
    /// Initializes the workload if this is the primary client (client_id = 0).
    /// Continuously attempts initialization until successful or a non-retryable error occurs.
    async fn setup(&mut self, db: Database) {
        if self.workload_context.client_id() != 0 {
            return;
        }

        loop {
            match self.workload_logic.init(&db, &self.workload_context).await {
                Ok(_) => {
                    self.workload_context.trace(
                        Severity::Info,
                        format!("{} initialized successfully", self.workload_logic.name()),
                        &[("Layer", "Rust"), ("Phase", "Setup")],
                    );

                    break;
                }
                Err(FdbBindingError::NonRetryableFdbError(_)) => {
                    self.workload_context.trace(
                        Severity::Warn,
                        format!("{} initialization failed", self.workload_logic.name()),
                        &[("Layer", "Rust"), ("Phase", "Setup")],
                    );

                    continue;
                }
                Err(_) => {
                    self.workload_context.trace(
                        Severity::Error,
                        format!(
                            "{} initialization failed on retryable error. Retrying...",
                            self.workload_logic.name()
                        ),
                        &[("Layer", "Rust"), ("Phase", "Setup")],
                    );

                    break;
                }
            }
        }
    }

    /// Runs the main workload simulation for the configured number of iterations.
    /// Tracks successful and failed iterations, and logs results.
    async fn start(&mut self, db: Database) {
        for iteration in 0..self.iterations {
            match self
                .workload_logic
                .simulate(&db, &self.workload_context)
                .await
            {
                Ok(_) => {
                    self.workload_context.trace(
                        Severity::Info,
                        format!("{} simulation successfully", self.workload_logic.name()),
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
                        format!("{} simulation failed", self.workload_logic.name()),
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
                        format!(
                            "{} simulation failed on retryable error. Retrying...",
                            self.workload_logic.name()
                        ),
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

    /// Verifies the workload results.
    /// Continuously attempts verification until successful or a non-retryable error occurs.
    async fn check(&mut self, db: Database) {
        loop {
            match self
                .workload_logic
                .verify(&db, &self.workload_context)
                .await
            {
                Ok(_) => {
                    println!("ok");
                    self.workload_context.trace(
                        Severity::Info,
                        format!("{} verification successfully", self.workload_logic.name()),
                        &[("Layer", "Rust"), ("Phase", "Check")],
                    );

                    break;
                }
                Err(FdbBindingError::NonRetryableFdbError(err)) => {
                    self.workload_context.trace(
                        Severity::Warn,
                        format!("{} verification failed", self.workload_logic.name()),
                        &[
                            ("Layer", "Rust"),
                            ("Phase", "Check"),
                            ("Error", &err.to_string()),
                        ],
                    );

                    continue;
                }
                Err(err) => {
                    let error = err.to_string();
                    self.workload_context.trace(
                        Severity::Error,
                        format!(
                            "{} verification failed on retryable error.",
                            self.workload_logic.name()
                        ),
                        &[("Layer", "Rust"), ("Phase", "Check"), ("Error", &error)],
                    );

                    break;
                }
            }
        }
    }

    /// Returns metrics about the workload execution, including successful and failed iterations.
    fn get_metrics(&self, mut out: Metrics) {
        out.push(Metric::val(
            "failed_iterations",
            self.failed_iterations as f64,
        ));
        out.push(Metric::val(
            "successful_iteration",
            self.successful_iteration as f64,
        ));
        self.workload_logic.extra_metrics(&mut out);
    }

    /// Returns the verification timeout in milliseconds.
    /// Uses workload logic override if provided, otherwise defaults to 5000ms.
    fn get_check_timeout(&self) -> f64 {
        self.workload_logic
            .override_check_timeout()
            .unwrap_or(5000.0)
    }
}
