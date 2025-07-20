use crate::stats_workload::StatsWorkload;
use crate::workload::Workload;
use foundationdb_simulation::{
    register_factory, RustWorkloadFactory, WorkloadContext, WrappedWorkload,
};

mod stats_workload;

mod workload;

struct CabinetSimulationFactory;

impl RustWorkloadFactory for CabinetSimulationFactory {
    fn create(name: String, context: WorkloadContext) -> WrappedWorkload {
        let iteration = context
            .get_option("iterations")
            .expect("Iteration option not found");
        match name.as_str() {
            stats_workload::STATS_WORKLOAD_NAME => {
                let stat_workload = StatsWorkload::new(&context);
                WrappedWorkload::new(Workload::new(context, iteration, stat_workload))
            }
            _ => panic!("Unknown workload: {}", name),
        }
    }
}

register_factory!(CabinetSimulationFactory);
