//! Binary entrypoint for the civ-watch dev harness.

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "civ_watch=info".into()),
        )
        .init();

    if let Err(e) = civ_watch::run().await {
        eprintln!("civ-watch: {e}");
        std::process::exit(1);
    }
}
