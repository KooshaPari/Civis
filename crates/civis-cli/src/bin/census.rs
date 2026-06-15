use std::time::Duration;

use civis_cli::census::{
    build_sim_status_request, decode_response, validate_sim_status, CensusConfig, CensusError,
    SimStatusResult,
};
use civis_cli::config::{census_config_from_env, load_dotenv};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Parser)]
#[command(
    name = "civis-census",
    about = "Query sim.status over the civ-server JSON-RPC WS bridge"
)]
struct Args {
    /// Override the WebSocket host (default $CIV_WS_HOST or 127.0.0.1).
    #[arg(long)]
    host: Option<String>,

    /// Override the WebSocket port (default $CIV_SERVER_PORT or 3000).
    #[arg(long)]
    port: Option<u16>,

    /// Override the WebSocket path (default $CIV_WS_PATH or /ws).
    #[arg(long)]
    path: Option<String>,

    /// Override the request timeout in milliseconds.
    #[arg(long)]
    timeout_ms: Option<u64>,

    /// Emit the raw JSON-RPC response rather than the parsed `sim.status`.
    #[arg(long)]
    raw: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    load_dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let base = census_config_from_env();
    let config = CensusConfig {
        host: args.host.unwrap_or(base.host),
        port: args.port.unwrap_or(base.port),
        path: args.path.unwrap_or(base.path),
        timeout_ms: args.timeout_ms.unwrap_or(base.timeout_ms),
    };

    let url = config.ws_url();
    let request_frame = build_sim_status_request(civ_server::jsonrpc::RequestId::Number(1));

    match census_call(&url, &request_frame, config.timeout()).await {
        Ok(text) => {
            if args.raw {
                println!("{text}");
                return;
            }
            let response = match decode_response(&text) {
                Ok(response) => response,
                Err(err) => {
                    eprintln!("civis-census: failed to decode response: {err:?}");
                    std::process::exit(2);
                }
            };
            match validate_sim_status(&response) {
                Ok(result) => print_result(&result),
                Err(err) => {
                    eprintln!("civis-census: {err}");
                    std::process::exit(3);
                }
            }
        }
        Err(err) => {
            eprintln!("civis-census: {err}");
            std::process::exit(1);
        }
    }
}

fn print_result(result: &SimStatusResult) {
    let payload = json!({
        "endpoint": "sim.status",
        "tick": result.tick,
        "population": result.population,
        "live": result.live,
    });
    match serde_json::to_string_pretty(&payload) {
        Ok(text) => println!("{text}"),
        Err(err) => {
            eprintln!("civis-census: failed to serialise result: {err}");
            std::process::exit(2);
        }
    }
}

async fn census_call(
    url: &str,
    request_frame: &str,
    request_timeout: Duration,
) -> Result<String, CensusCallError> {
    let connect =
        tokio_tungstenite::connect_async(url)
            .await
            .map_err(|err| CensusCallError::Connect {
                url: url.to_string(),
                source: err,
            })?;
    let (mut ws, _response) = connect;

    ws.send(Message::Text(request_frame.to_string()))
        .await
        .map_err(|err| CensusCallError::Send { source: err })?;

    let frame = timeout(request_timeout, ws.next())
        .await
        .map_err(|_| CensusCallError::Timeout {
            url: url.to_string(),
            timeout_ms: request_timeout.as_millis() as u64,
        })?
        .ok_or(CensusCallError::Closed)?
        .map_err(CensusCallError::Transport)?;

    let text = match frame {
        Message::Text(text) => text,
        Message::Binary(bytes) => {
            String::from_utf8(bytes).map_err(|err| CensusCallError::Decode { source: err })?
        }
        Message::Close(_) => return Err(CensusCallError::Closed),
        other => {
            return Err(CensusCallError::UnexpectedFrame {
                kind: format!("{other:?}"),
            })
        }
    };

    let _ = ws.close(None).await;
    Ok(text)
}

#[derive(Debug, thiserror::Error)]
enum CensusCallError {
    /// Failed to open the WebSocket connection to the server.
    #[error("civ-server WS connect to {url} failed: {source}")]
    Connect {
        /// Target URL.
        url: String,
        /// Underlying tungstenite error.
        #[source]
        source: tokio_tungstenite::tungstenite::Error,
    },
    /// The request text frame could not be sent.
    #[error("send sim.status request failed: {source}")]
    Send {
        /// Underlying tungstenite error.
        #[source]
        source: tokio_tungstenite::tungstenite::Error,
    },
    /// No frame arrived before the per-request timeout fired.
    #[error("civ-server at {url} did not respond within {timeout_ms}ms")]
    Timeout {
        /// Target URL.
        url: String,
        /// Configured timeout in milliseconds.
        timeout_ms: u64,
    },
    /// The server closed the connection before sending a frame.
    #[error("civ-server closed the WS connection before sending sim.status")]
    Closed,
    /// The transport raised a non-fatal error mid-stream.
    #[error("civ-server WS transport error: {0}")]
    Transport(#[source] tokio_tungstenite::tungstenite::Error),
    /// The server sent a binary frame that was not valid UTF-8.
    #[error("sim.status binary frame was not valid UTF-8: {source}")]
    Decode {
        /// Underlying UTF-8 error.
        #[source]
        source: std::string::FromUtf8Error,
    },
    /// The server sent an unexpected WebSocket message type.
    #[error("civ-server sent an unexpected frame: {kind}")]
    UnexpectedFrame {
        /// Human-readable frame kind (e.g. `Ping`).
        kind: String,
    },
}

impl From<CensusError> for CensusCallError {
    fn from(err: CensusError) -> Self {
        CensusCallError::UnexpectedFrame {
            kind: format!("{err:?}"),
        }
    }
}
