# observability

> OpenTelemetry OTLP export pipeline and Prometheus scrape endpoint for Civis.

## Overview

The civ-observability crate wires up OpenTelemetry tracing and metrics export for all Civis services. It provides an OTLP gRPC pipeline for traces and metrics, and a Prometheus HTTP scrape endpoint for dashboards and alerting. Tracing spans are bridged through tracing-opentelemetry.

## Features

- OTLP gRPC export for traces and metrics (OpenTelemetry 0.32)
- Prometheus scrape endpoint with configurable registry
- Tracing-subscriber integration via tracing-opentelemetry
- Tokio multi-threaded runtime for non-blocking export
- Configurable batch spans and metrics export intervals

## Usage

```rust
use civ_observability::init;

// Initialize the OTLP pipeline and Prometheus endpoint
let guard = init::setup("civis-service", "0.1.0")?;

// Use tracing spans as usual
tracing::info_span!("simulation_tick").in_scope(|| {
    // ...
});

// Metrics are exported automatically
guard.shutdown()?;
```

## Architecture

The setup function initializes an OpenTelemetry TracerProvider and MeterProvider configured with OTLP gRPC exporters. A Prometheus registry is created and bound to a hyper HTTP server for scraping. The tracing-opentelemetry layer bridges Rust tracing spans into OpenTelemetry traces.

## License

Part of the Civis project.
