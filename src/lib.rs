use crate::errors::CabinetError;
use crate::server::CabinetServer;
use cabinet_lib::foundationdb::Database;
use clap::Parser;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime::Tokio, trace, Resource};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod errors;
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
    init_tracing(&args);

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

    Ok(())
}

/// Initialize tracing with the provided configuration
fn init_tracing(args: &Args) {
    // If no tracing endpoint is provided, use the default fmt subscriber
    if args.tracing_endpoint.is_none() {
        tracing_subscriber::fmt::init();
        return;
    }

    // Configure tracing with the provided endpoint and authentication
    if let Some(endpoint) = &args.tracing_endpoint {
        // Create a resource with service information
        let resource = Resource::new(vec![
            KeyValue::new("service.name", "cabinet-server"),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]);

        // Configure the OTLP exporter
        let mut otlp_exporter = opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(endpoint);

        // Add authentication if provided
        if let Some(auth) = &args.tracing_auth {
            // Add the authentication token as a header
            // This typically uses the "Authorization" header with a "Bearer" prefix
            let mut headers = std::collections::HashMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {}", auth));
            otlp_exporter = otlp_exporter.with_headers(headers);
        }

        // Create a tracer provider with the configured exporter
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .with_trace_config(trace::config().with_resource(resource))
            .install_batch(Tokio)
            .expect("Failed to install OpenTelemetry tracer");

        // Create an OpenTelemetry tracing layer
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        // Create a formatting layer for console output
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .with_target(true);

        // Use the tracing subscriber registry to combine multiple layers
        tracing_subscriber::registry()
            .with(telemetry)
            .with(fmt_layer)
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        info!(
            "OpenTelemetry tracing initialized with endpoint: {}",
            endpoint
        );
    }
}
