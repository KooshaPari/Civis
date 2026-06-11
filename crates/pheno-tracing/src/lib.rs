use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize process-wide tracing with one EnvFilter policy for every Civis crate.
///
/// The default filter is `info`; set `RUST_LOG` to override it. Production-like
/// environments emit JSON, while local/dev/test runs use pretty text output.
pub fn init() {
    let _ = try_init();
}

pub fn try_init() -> Result<(), tracing_subscriber::util::TryInitError> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(env_filter);

    #[cfg(feature = "opentelemetry")]
    let registry = {
        let otlp_layer = otlp_layer();
        registry.with(otlp_layer)
    };

    if use_json_output() {
        registry.with(fmt::layer().json()).try_init()
    } else {
        registry.with(fmt::layer().pretty()).try_init()
    }
}

fn use_json_output() -> bool {
    ["CIVIS_ENV", "APP_ENV", "RUST_ENV"]
        .iter()
        .filter_map(|key| std::env::var(key).ok())
        .any(|value| matches!(value.as_str(), "production" | "prod"))
}

#[cfg(feature = "opentelemetry")]
fn otlp_layer<S>() -> tracing_opentelemetry::OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    use opentelemetry_otlp::WithExportConfig;

    let exporter = opentelemetry_otlp::new_exporter().tonic();
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("install OTLP tracing pipeline");

    tracing_opentelemetry::layer().with_tracer(tracer)
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_works() {
        super::init();
        super::init();
    }
}
