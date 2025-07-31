use crate::errors::CabinetError;
use crate::server::CabinetServer;
use cabinet_lib::foundationdb::Database;
use std::sync::Arc;
use tracing::{error, info};

mod errors;
mod server;
mod state;

#[tracing::instrument]
pub async fn run() -> Result<(), CabinetError> {
    tracing_subscriber::fmt::init();
    info!("Starting up...");
    info!("Getting network thread...");
    let _guard = toolbox::get_network_thread()?;
    info!("Network thread acquired");

    let fdb_cluster_path = std::env::var("FDB_CLUSTER_PATH".to_string()).ok();

    info!(?fdb_cluster_path, "Acquiring database...");
    let database = Database::new_compat(fdb_cluster_path.as_deref())
        .await
        .expect("Failed to create database");
    info!("Database acquired");

    let database = Arc::new(database);

    // Start the TCP server in a separate task
    info!("Starting TCP server...");
    let mut server = CabinetServer::new("0.0.0.0:8080");
    if let Err(e) = server.start(database).await {
        error!("TCP server error: {}", e);
    }

    Ok(())
}
