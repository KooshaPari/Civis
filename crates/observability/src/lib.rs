//! OpenTelemetry observability setup — OTLP trace export and Prometheus metrics endpoint.

pub mod otel;

pub use otel::{init_observability, ObservabilityConfig};
