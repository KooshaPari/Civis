//! OpenTelemetry observability setup — OTLP trace export and Prometheus metrics endpoint.
#![forbid(unsafe_code)]

pub mod otel;
#[cfg(test)]
pub mod otlp_validation;
pub mod perf;

pub use otel::{init_observability, ObservabilityConfig, ObservabilityError};
pub use perf::{SimMetricSnapshot, SimMetrics};
