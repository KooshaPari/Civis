//! `civis-mcp` binary — stdio MCP transport.
//!
//! Wires `CivisMcpServer` to rmcp's stdio transport and waits for graceful
//! shutdown. All the actual tool logic lives in [`server`] and [`lib`].

use rmcp::transport::stdio;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() {
    let service = civis_mcp::CivisMcpServer::new().serve(stdio()).await;
    match service {
        Ok(service) => {
            if let Err(err) = service.waiting().await {
                eprintln!("civis-mcp server terminated: {err}");
            }
        }
        Err(err) => {
            eprintln!("civis-mcp failed to start: {err}");
            std::process::exit(1);
        }
    }
}
