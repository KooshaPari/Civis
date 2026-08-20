//! Full OTLP export pipeline and Prometheus scrape endpoint.
//!
//! Call [`init_observability`] early in `main()` to:
//!
//! 1. Install an OTLP trace exporter that ships spans to a collector at
//!    `OTEL_EXPORTER_OTLP_ENDPOINT` (default `http://localhost:4317`).
//! 2. Spin up an HTTP server on `prometheus_port` (default **9090**) that
//!    serves Prometheus‑scrapeable metrics in text format.

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::SdkTracerProvider;

/// Configuration for the unified observability stack.
pub struct ObservabilityConfig {
    /// `service.name` resource attribute attached to every span.
    pub service_name: String,

    /// OTLP gRPC collector endpoint.  Falls back to the
    /// `OTEL_EXPORTER_OTLP_ENDPOINT` env var, then `http://localhost:4317`.
    pub otlp_endpoint: Option<String>,

    /// Port for the Prometheus scrape HTTP server.  Defaults to **9090**.
    pub prometheus_port: Option<u16>,
}

/// Initialise OTLP tracing + Prometheus metrics and return the
/// [`SdkTracerProvider`] so callers can obtain a [`Tracer`](opentelemetry::trace::Tracer).
pub fn init_observability(config: ObservabilityConfig) -> SdkTracerProvider {
    // ── OTLP trace export ────────────────────────────────────────────────
    let endpoint = config.otlp_endpoint.unwrap_or_else(|| {
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string())
    });

    let tracer_provider: SdkTracerProvider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", config.service_name),
                ])),
        )
        .install_batch(runtime::Tokio)
        .expect("Failed to install OTLP tracer");

    // ── Prometheus metrics endpoint ──────────────────────────────────────
    let prometheus_port = config.prometheus_port.unwrap_or(9090);

    // `install_recorder` registers an OTel → Prometheus bridge in the
    // global prometheus registry so `prometheus::gather()` returns OTel
    // metrics alongside any manually‑registered ones.
    let exporter = opentelemetry_prometheus::exporter()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    // Spawn a dedicated thread with its own Tokio runtime so the scrape
    // server never competes with the main application runtime.
    std::thread::spawn(move || {
        // Prevent the exporter (and the metrics reader it holds) from being
        // dropped — the `move` closure keeps it alive for the thread's
        // lifetime.
        let _keep_alive = exporter;

        let rt = tokio::runtime::Runtime::new().expect("Failed to create prometheus runtime");
        rt.block_on(async {
            use prometheus::{Encoder, TextEncoder};
            use tokio::io::AsyncWriteExt;
            use tokio::net::TcpListener;

            let addr = ([0, 0, 0, 0], prometheus_port);
            let listener = TcpListener::bind(addr)
                .await
                .expect("Failed to bind Prometheus metrics listener");

            println!("Prometheus metrics server listening on :{prometheus_port}");

            loop {
                match listener.accept().await {
                    Ok((mut stream, _peer)) => {
                        let encoder = TextEncoder::new();
                        let metric_families = prometheus::gather();
                        let mut buffer = Vec::new();
                        let _ = encoder.encode(&metric_families, &mut buffer);

                        let content_type = encoder.format_type();
                        let header = format!(
                            "HTTP/1.1 200 OK\r\n\
                             Content-Type: {content_type}\r\n\
                             Content-Length: {}\r\n\
                             Connection: close\r\n\r\n",
                            buffer.len(),
                        );
                        let mut response = header.into_bytes();
                        response.extend_from_slice(&buffer);
                        let _ = stream.write_all(&response).await;
                    }
                    Err(err) => {
                        eprintln!("Prometheus listener accept error: {err}");
                    }
                }
            }
        });
    });

    println!("OpenTelemetry observability initialized (OTLP + Prometheus)");

    tracer_provider
}
