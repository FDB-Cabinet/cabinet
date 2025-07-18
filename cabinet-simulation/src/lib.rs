use crate::stats_workload::StatsWorkload;
use foundationdb_simulation::{
    register_factory, RustWorkloadFactory, WorkloadContext, WrappedWorkload,
};

mod stats_workload;

struct CabinetSimulationFactory;

impl RustWorkloadFactory for CabinetSimulationFactory {
    fn create(name: String, context: WorkloadContext) -> WrappedWorkload {
        let iteration = context
            .get_option("iterations")
            .expect("Iteration option not found");
        match name.as_str() {
            stats_workload::STATS_WORKLOAD_NAME => {
                WrappedWorkload::new(StatsWorkload::new(context, iteration))
            }
            _ => panic!("Unknown workload: {}", name),
        }
    }
}

register_factory!(CabinetSimulationFactory);
