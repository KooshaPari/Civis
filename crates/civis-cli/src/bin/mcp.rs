//! `civis-mcp` — a tiny JSON-RPC 2.0 over stdio shim that exposes the
//! `civis-cli` subcommands as MCP-shaped `tools/call` methods.
//!
//! The full MCP server lives in `crates/civis-mcp` (uses `rmcp`). This
//! process is a deliberately smaller fallback for toolchains that cannot
//! compile `rmcp` (it has a much larger transitive dep set). The wire
//! shape is one JSON object per line, with the standard `jsonrpc`, `id`,
//! `method`, and `params` fields. Supported methods:
//!
//! | Method          | Params                                           | Returns |
//! |-----------------|--------------------------------------------------|---------|
//! | `tools/list`    | none                                             | `{ tools: [...] }` |
//! | `tools/call`    | `{ name, arguments }`                            | result of the inner subcommand as JSON |
//!
//! `tools/call` dispatches to the matching `civis-cli` library helper and
//! returns its output verbatim (no `cargo run` shell-out — the bin imports
//! the library so the harness stays single-process).

use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use civis_cli::census::{
    build_sim_status_request, decode_response, validate_sim_status, CensusConfig,
};
use civis_cli::config::{census_config_from_env, load_dotenv};
use civis_cli::pixels::{compute_pixel_stats, sample_rgb_grid};

#[derive(Debug, Serialize, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

impl RpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }
    fn failure(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

fn main() -> io::Result<()> {
    load_dotenv();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let lines = stdin.lock().lines();

    for line in lines {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => handle(req),
            Err(err) => RpcResponse::failure(Value::Null, -32700, format!("Parse error: {err}")),
        };
        let serialised = serde_json::to_string(&response)
            .map_err(|err| io::Error::other(format!("serialise response: {err}")))?;
        writeln!(stdout, "{serialised}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle(req: RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        "tools/list" => RpcResponse::success(
            req.id,
            json!({
                "tools": [
                    { "name": "verify", "description": "Run Bevy frame capture (Bevy feature only). Returns the output path." },
                    { "name": "pixels", "description": "Compute pixel statistics for a PNG file. { path, grid? }" },
                    { "name": "census", "description": "Read sim.status over the civ-server JSON-RPC WS bridge." },
                ]
            }),
        ),
        "tools/call" => match req.params.get("name").and_then(Value::as_str) {
            Some("pixels") => match pixels_tool(&req.params) {
                Ok(value) => RpcResponse::success(req.id, value),
                Err(message) => RpcResponse::failure(req.id, -32602, message),
            },
            Some("census") => match block_on(census_tool(&req.params)) {
                Ok(value) => RpcResponse::success(req.id, value),
                Err(message) => RpcResponse::failure(req.id, -32603, message),
            },
            Some("verify") => RpcResponse::failure(
                req.id,
                -32601,
                "verify tool requires the `bevy` feature; rebuild with `cargo run -p civis-cli --features bevy --bin civis-mcp`",
            ),
            Some(other) => RpcResponse::failure(
                req.id,
                -32601,
                format!("unknown tool `{other}`"),
            ),
            None => RpcResponse::failure(req.id, -32602, "missing `name`"),
        },
        other => RpcResponse::failure(
            req.id,
            -32601,
            format!("method `{other}` is not implemented"),
        ),
    }
}

fn pixels_tool(params: &Value) -> Result<Value, String> {
    let path = params
        .get("arguments")
        .and_then(|a| a.get("path"))
        .and_then(Value::as_str)
        .ok_or_else(|| "missing `path` argument".to_string())?;
    let grid = params
        .get("arguments")
        .and_then(|a| a.get("grid"))
        .and_then(Value::as_u64)
        .map(|g| g as usize)
        .unwrap_or(16);
    let bytes = std::fs::read(path).map_err(|err| format!("read {path}: {err}"))?;
    let decoder = png::Decoder::new(&bytes[..]);
    let mut reader = decoder
        .read_info()
        .map_err(|err| format!("png decode {path}: {err}"))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|err| format!("png frame {path}: {err}"))?;
    let data: Vec<u8> = match frame.color_type {
        png::ColorType::Rgb => buf[..frame.buffer_size()].to_vec(),
        png::ColorType::Rgba => buf[..frame.buffer_size()]
            .chunks_exact(4)
            .flat_map(|px| [px[0], px[1], px[2]])
            .collect(),
        png::ColorType::Grayscale => buf[..frame.buffer_size()]
            .iter()
            .flat_map(|&v| [v, v, v])
            .collect(),
        png::ColorType::GrayscaleAlpha => buf[..frame.buffer_size()]
            .chunks_exact(2)
            .flat_map(|px| [px[0], px[0], px[0]])
            .collect(),
        png::ColorType::Indexed => {
            return Err("indexed PNGs are not supported".to_string());
        }
    };
    let samples = sample_rgb_grid(frame.width as usize, frame.height as usize, grid, &data);
    let stats = compute_pixel_stats(&samples);
    Ok(json!({ "path": path, "grid": grid, "stats": stats }))
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("civis-mcp runtime")
        .block_on(future)
}

async fn census_tool(_params: &Value) -> Result<Value, String> {
    let config: CensusConfig = census_config_from_env();
    let url = config.ws_url();
    let frame = build_sim_status_request(civ_server::jsonrpc::RequestId::Number(1));
    let connect = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|err| format!("WS connect {url}: {err}"))?;
    let (mut ws, _response) = connect;
    use futures_util::{SinkExt, StreamExt};
    ws.send(tokio_tungstenite::tungstenite::Message::Text(frame))
        .await
        .map_err(|err| format!("WS send sim.status: {err}"))?;
    let response = tokio::time::timeout(config.timeout(), ws.next())
        .await
        .map_err(|_| format!("timed out waiting for {url}"))?
        .ok_or_else(|| "server closed the connection".to_string())?
        .map_err(|err| format!("WS read: {err}"))?;
    let text = match response {
        tokio_tungstenite::tungstenite::Message::Text(t) => t,
        tokio_tungstenite::tungstenite::Message::Binary(b) => {
            String::from_utf8(b).map_err(|err| format!("binary frame utf-8: {err}"))?
        }
        other => return Err(format!("unexpected frame {other:?}")),
    };
    let parsed = decode_response(&text).map_err(|err| format!("decode: {err:?}"))?;
    let result = validate_sim_status(&parsed).map_err(|err| format!("validate: {err}"))?;
    serde_json::to_value(result).map_err(|err| format!("serialise: {err}"))
}
