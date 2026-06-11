//! Binary entrypoint for the civ-watch dev harness.

#[tokio::main]
async fn main() {
    pheno_tracing::init();

    civ_watch::run().await;
}
