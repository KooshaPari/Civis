//! OTLP observability demo.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p civ-observability --example otlp_demo
//! ```
//!
//! Set `OTEL_EXPORTER_OTLP_ENDPOINT` to point at a running OTLP collector
//! (default `http://localhost:4317`).  The Prometheus scrape endpoint is
//! served on port 9090.

use civ_observability::{init_observability, ObservabilityConfig};
use opentelemetry::trace::TracerProvider as _;
use prometheus::{Encoder, TextEncoder};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    // ------------------------------------------------------------------
    // 1. Initialise the full observability stack.
    // ------------------------------------------------------------------
    println!("=== Civis OTLP Observability Demo ===\n");

    let provider = init_observability(ObservabilityConfig {
        service_name: "civis-otlp-demo".to_string(),
        otlp_endpoint: None,   // reads OTEL_EXPORTER_OTLP_ENDPOINT
        prometheus_port: None, // default 9090
    })
    .expect("Failed to initialise observability");

    let tracer = provider.tracer("civis-demo");

    // Install tracing subscriber with both console output and OTel layer.
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .try_init();

    println!(
        "Observability stack initialised.\n\
         OTLP endpoint : {}\n\
         Prometheus    : http://0.0.0.0:9090/metrics\n",
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string()),
    );

    // ------------------------------------------------------------------
    // 2. Record Prometheus metrics.
    // ------------------------------------------------------------------
    let request_counter = prometheus::register_counter!(
        "civis_demo_requests_total",
        "Total requests processed by the demo"
    )
    .expect("failed to register counter");

    let response_histogram = prometheus::register_histogram!(
        "civis_demo_response_seconds",
        "Response latency in seconds"
    )
    .expect("failed to register histogram");

    for i in 0..5 {
        request_counter.inc();
        response_histogram.observe(0.01 + (i as f64) * 0.005);
    }

    println!("Recorded 5 requests and latency samples.");

    // Show that Prometheus can gather the metrics we just recorded.
    let families = prometheus::gather();
    let encoder = TextEncoder::new();
    let mut buf = Vec::new();
    encoder.encode(&families, &mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();
    println!(
        "\n--- Prometheus metrics ({} bytes) ---\n{}",
        text.len(),
        // Print only the lines belonging to our demo counters.
        text.lines()
            .filter(|l| l.contains("civis_demo_"))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    // ------------------------------------------------------------------
    // 3. Create traced spans via the OTel layer.
    // ------------------------------------------------------------------
    println!("\n--- Traced spans ---");

    let span = tracing::info_span!("game_tick", tick = 1, players = 4);
    let _guard = span.enter();
    tracing::info!("Simulating game tick");
    drop(_guard);

    {
        let _span = tracing::info_span!("network_broadcast", clients = 12).entered();
        tracing::debug!("Broadcasting state to all connected clients");
    }

    println!("Two spans created and queued for export.\n");

    // ------------------------------------------------------------------
    // 4. Flush the OTLP exporter and shut down.
    // ------------------------------------------------------------------
    provider
        .shutdown()
        .expect("failed to shut down tracer provider");

    println!("OTLP exporter flushed. Demo complete.");
}
