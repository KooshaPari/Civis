//! Validation tests for the OTLP observability pipeline.
//!
//! These tests verify that the full OpenTelemetry stack (OTLP span export,
//! Prometheus metrics, tracing integration) can be initialized and used
//! correctly without requiring a live collector.

use crate::{init_observability, ObservabilityConfig};
use opentelemetry::trace::Span as _;
use opentelemetry::trace::Tracer as _;
use opentelemetry::trace::TracerProvider as _;
use prometheus::{Encoder, TextEncoder};

/// Initialise the observability stack with a test-specific config.
///
/// Uses a dummy OTLP endpoint (no collector required) and a high
/// Prometheus port to avoid collisions with any running services.
fn setup_test_observability() -> opentelemetry_sdk::trace::SdkTracerProvider {
    init_observability(ObservabilityConfig {
        service_name: "civis-otlp-validation-test".to_string(),
        // The span exporter builder succeeds even when no collector is listening;
        // spans simply fail to deliver (which is fine for a compile-and-init test).
        otlp_endpoint: Some("http://127.0.0.1:4318".to_string()),
        prometheus_port: Some(19876),
    })
    .expect("Failed to initialise test observability")
}

// ---------------------------------------------------------------------------
// Test: OTLP pipeline initialises without errors
// ---------------------------------------------------------------------------

#[test]
fn otlp_pipeline_initialises() {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let _guard = rt.enter();

    let provider = setup_test_observability();
    let tracer = provider.tracer("otlp-validation");
    // Creating a span should not panic even though no collector is listening.
    let mut span = tracer.start("validation-span");
    span.set_status(opentelemetry::trace::Status::Ok);
    span.end();
    // Shut down the provider so the background batch worker flushes.
    let _ = provider.shutdown();
}

// ---------------------------------------------------------------------------
// Test: Prometheus metrics can be gathered and serialised
// ---------------------------------------------------------------------------

#[test]
fn prometheus_metrics_gathered() {
    // Register a test-specific counter so we don't pollute the global
    // registry with counters from other tests.
    let counter = prometheus::register_counter!(
        "civis_otlp_validation_total",
        "Validation test counter"
    )
    .expect("failed to register test counter");
    counter.inc_by(7.0);

    let histogram = prometheus::register_histogram!(
        "civis_otlp_validation_latency_seconds",
        "Validation test histogram"
    )
    .expect("failed to register test histogram");
    histogram.observe(0.042);

    let metric_families = prometheus::gather();
    assert!(
        !metric_families.is_empty(),
        "prometheus::gather() should return at least the registered metrics"
    );

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("failed to encode metrics");

    let text = String::from_utf8(buffer).expect("metrics output is not valid UTF-8");
    assert!(
        text.contains("civis_otlp_validation_total"),
        "serialised output must contain our test counter"
    );
    assert!(
        text.contains("civis_otlp_validation_latency_seconds"),
        "serialised output must contain our test histogram"
    );
}

// ---------------------------------------------------------------------------
// Test: Tracing spans can be created and exported through OTel layer
// ---------------------------------------------------------------------------

#[test]
fn tracing_spans_created_and_exported() {
    use opentelemetry::trace::TracerProvider as _;
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let _guard = rt.enter();

    let provider = setup_test_observability();
    let tracer = provider.tracer("civis-tracing-test");

    // `try_init` is safe to call multiple times across tests — it returns
    // `Err` if a global subscriber is already installed, which is fine.
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .with(OpenTelemetryLayer::new(tracer))
        .try_init();

    // Create a span using the `tracing` macro — this exercises the full
    // tracing -> OpenTelemetry bridge.
    let _guard = tracing::info_span!("otlp_validation_span", tick = 42).entered();
    tracing::info!("OTLP validation event inside span");
    drop(_guard);

    let _ = provider.shutdown();
}

// ---------------------------------------------------------------------------
// Test: OpenTelemetryLayer is properly configured
// ---------------------------------------------------------------------------

#[test]
fn otel_layer_properly_configured() {
    use opentelemetry::trace::TracerProvider as _;
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::layer::SubscriberExt;

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let _guard = rt.enter();

    let provider = setup_test_observability();
    let tracer = provider.tracer("civis-layer-test");

    // Build the layer and verify it wraps a working tracer by creating a
    // span through the combined subscriber stack.
    let layer = OpenTelemetryLayer::new(tracer);
    let subscriber = tracing_subscriber::registry().with(layer);

    // Dispatch a tracing event through the layered subscriber to confirm
    // the OTel layer doesn't panic on span creation.
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("layer_config_test");
        let _enter = span.enter();
        tracing::info!("message dispatched through OTel layer");
    });

    let _ = provider.shutdown();
}
