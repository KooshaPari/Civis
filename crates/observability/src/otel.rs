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
pub fn init_observability(config: ObservabilityConfig) -> SdkTracerProvider {
    let endpoint = config.otlp_endpoint.unwrap_or_else(|| {
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string())
    });

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to build OTLP span exporter");

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
        .expect("Failed to build Prometheus exporter");

    std::thread::spawn(move || {
        let _keep_alive = prom_exporter;
        let rt = tokio::runtime::Runtime::new().expect("Failed to create prometheus runtime");
        rt.block_on(async {
            use prometheus::{Encoder, TextEncoder};
            use std::net::IpAddr;
            use tokio::io::AsyncWriteExt;
            use tokio::net::TcpListener;

            let addr: (IpAddr, u16) = (
                IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                prometheus_port,
            );
            let listener = TcpListener::bind(addr)
                .await
                .expect("Failed to bind Prometheus metrics listener");

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

    tracer_provider
}
