use crate::errors::CabinetError;
use crate::instrumentation::init_tracing;
use crate::server::CabinetServer;
use cabinet_lib::foundationdb::Database;
use clap::Parser;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

mod errors;
mod instrumentation;
mod server;
mod state;
#[cfg(test)]
mod tests;

/// Cabinet server with configurable tracing
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    pub address: String,

    /// Tracing endpoint URL (e.g., http://localhost:4317 for OTLP)
    #[arg(long)]
    pub tracing_endpoint: Option<String>,

    /// Tracing authentication token or header
    #[arg(long)]
    pub tracing_auth: Option<String>,
}

#[tracing::instrument]
pub async fn run() -> Result<(), CabinetError> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing with custom configuration if provided
    let otel_guard = init_tracing(&args);

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
    let mut server = CabinetServer::new(&args.address);
    if let Err(e) = server.start(database).await {
        error!("TCP server error: {}", e);
    }

    drop(otel_guard);

    Ok(())
}
