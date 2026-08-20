# civis-mcp

> MCP (Model Context Protocol) server exposing 30+ verification and interaction tools over JSON-RPC.

## Overview

The `civis-mcp` crate implements an MCP server that acts as the bridge between AI agents and the Civis simulation. It exposes over 30 tools covering census queries, snapshot capture, emergence dashboard access, god-mode actions, diplomacy management, research progression, and save/load operations.

Under the hood, every tool is a thin proxy that translates MCP tool calls into `civ-server` JSON-RPC requests over WebSocket. This keeps the MCP layer stateless and the simulation state authoritative in the server process.

The tool definitions are structured for easy discovery by MCP clients, with typed parameters, descriptions, and schema metadata that enable AI agents to autonomously explore and interact with the simulation.

## Features

- 30+ MCP tools covering simulation read and write operations
- Census, snapshot, and emergence dashboard tools
- God-mode actions for direct simulation intervention
- Diplomacy and research management tools
- Save/load and state management operations
- Thin JSON-RPC proxy over WebSocket to `civ-server`
- Typed tool definitions with schema metadata for AI agent discovery

## Usage

```rust
use civis_mcp::*;
```

## Architecture

- **Tool definitions** — Structured metadata for each of the 30+ available MCP tools
- **dispatch_rpc_method** — Central dispatcher that routes MCP tool calls to the appropriate JSON-RPC method
- **WebSocket transport** — Async connection to `civ-server` for all simulation I/O

The MCP server is a thin adapter layer — it holds no simulation state, validates tool parameters against schemas, and forwards calls to the server process.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
