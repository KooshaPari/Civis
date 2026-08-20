//! OpenTelemetry observability setup — OTLP trace export and Prometheus metrics endpoint.
#![forbid(unsafe_code)]

pub mod otel;

pub use otel::{init_observability, ObservabilityConfig};
