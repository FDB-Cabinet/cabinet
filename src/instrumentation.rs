use crate::Args;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::attribute::{SERVICE_NAME, SERVICE_VERSION};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use std::collections::HashMap;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn resource() -> Resource {
    Resource::builder()
        .with_service_name(env!("CARGO_PKG_NAME"))
        .with_schema_url(
            [
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            ],
            SCHEMA_URL,
        )
        .build()
}

// Construct TracerProvider for OpenTelemetryLayer
fn init_tracer_provider(args: &Args) -> SdkTracerProvider {
    let endpoint = args
        .tracing_endpoint
        .as_ref()
        .expect("Missing tracing endpoint.");

    let mut exporter_builder = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(endpoint);

    if let Some(auth) = args.tracing_auth.as_ref() {
        let headers = HashMap::from([("Authorization".to_string(), auth.to_string())]);

        exporter_builder = exporter_builder.with_headers(headers);
    }

    let exporter = exporter_builder.build().unwrap();

    SdkTracerProvider::builder()
        // Customize sampling strategy
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            1.0,
        ))))
        .with_resource(resource())
        .with_batch_exporter(exporter)
        .build()
}

// Initialize tracing-subscriber and return OtelGuard for opentelemetry-related termination processing
pub fn init_tracing(args: &Args) -> OtelGuard {
    if args.tracing_endpoint.is_some() {
        let tracer_provider = init_tracer_provider(args);

        let tracer = tracer_provider.tracer("tracing-otel-subscriber");

        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .with(OpenTelemetryLayer::new(tracer))
            .init();

        return OtelGuard {
            tracer_provider: Some(tracer_provider),
        };
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    OtelGuard {
        tracer_provider: None,
    }
}

pub struct OtelGuard {
    tracer_provider: Option<SdkTracerProvider>,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        println!("Shutting down OpenTelemetry...");
        if let Some(tracer) = &self.tracer_provider {
            if let Err(err) = tracer.shutdown() {
                eprintln!("{err:?}");
            }
        }
    }
}
