//! Full OTLP export pipeline and Prometheus scrape endpoint.
//!
//! Call [`init_observability`] early in `main()` to:
//!
//! 1. Install an OTLP trace exporter that ships spans to a collector at
//!    `OTEL_EXPORTER_OTLP_ENDPOINT` (default `http://localhost:4317`).
//! 2. Spin up an HTTP server on `prometheus_port` (default **9090**) that
//!    serves Prometheus-scrapeable metrics in text format.

use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::fmt;

/// Errors that can occur during observability initialisation.
#[derive(Debug)]
pub enum ObservabilityError {
    /// OTLP span exporter could not be built.
    OtlpExporter(String),
    /// Prometheus exporter could not be built.
    PrometheusExporter(String),
    /// Tokio runtime for Prometheus could not be created.
    PrometheusRuntime(String),
    /// Prometheus metrics listener could not bind.
    PrometheusBind(String),
}

impl fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObservabilityError::OtlpExporter(msg) => write!(f, "OTLP exporter build failed: {msg}"),
            ObservabilityError::PrometheusExporter(msg) => write!(f, "Prometheus exporter build failed: {msg}"),
            ObservabilityError::PrometheusRuntime(msg) => write!(f, "Prometheus runtime build failed: {msg}"),
            ObservabilityError::PrometheusBind(msg) => write!(f, "Prometheus metrics listener bind failed: {msg}"),
        }
    }
}

impl std::error::Error for ObservabilityError {}

/// Configuration for the unified observability stack.
pub struct ObservabilityConfig {
    /// `service.name` resource attribute attached to every span.
    pub service_name: String,
    /// OTLP gRPC collector endpoint.
    pub otlp_endpoint: Option<String>,
    /// Port for the Prometheus scrape HTTP server.
    pub prometheus_port: Option<u16>,
}

/// Initialise OTLP tracing + Prometheus metrics and return the
/// [`SdkTracerProvider`] so callers can obtain a Tracer.
pub fn init_observability(config: ObservabilityConfig) -> Result<SdkTracerProvider, ObservabilityError> {
    let endpoint = config.otlp_endpoint.unwrap_or_else(|| {
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string())
    });

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| ObservabilityError::OtlpExporter(e.to_string()))?;

    let tracer_provider: SdkTracerProvider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder_empty()
                .with_service_name(config.service_name)
                .build(),
        )
        .build();

    let prometheus_port = config.prometheus_port.unwrap_or(9090);

    let prom_exporter = opentelemetry_prometheus::exporter()
        .build()
        .map_err(|e| ObservabilityError::PrometheusExporter(e.to_string()))?;

    std::thread::spawn(move || {
        let _keep_alive = prom_exporter;
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => { tracing::warn!("Prometheus runtime failed: {e}"); return; }
        };
        rt.block_on(async {
            use prometheus::{Encoder, TextEncoder};
            use std::net::IpAddr;
            use tokio::io::AsyncWriteExt;
            use tokio::net::TcpListener;

            let addr: (IpAddr, u16) = (
                IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                prometheus_port,
            );
            let listener = match TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => { tracing::warn!("Prometheus bind failed: {e}"); return; }
            };

            loop {
                match listener.accept().await {
                    Ok((mut stream, _peer)) => {
                        let encoder = TextEncoder::new();
                        let metric_families = prometheus::gather();
                        let mut buffer = Vec::new();
                        let _ = encoder.encode(&metric_families, &mut buffer);
                        let content_type = encoder.format_type();
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            buffer.len(),
                        );
                        let mut response = header.into_bytes();
                        response.extend_from_slice(&buffer);
                        let _ = stream.write_all(&response).await;
                    }
                    Err(e) => {
                        eprintln!("Prometheus accept error: {e}");
                    }
                }
            }
        });
    });

    Ok(tracer_provider)
}
