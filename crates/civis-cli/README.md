# civis-cli

> Programmatic verification harness for Civis — pixel validation, entity census, and regression dumping.

## Overview

The `civis-cli` crate provides the command-line tooling and library functions for automated verification of Civis builds. It is designed for CI pipelines, golden-image testing, and runtime sanity checks, operating against a running Civis instance over WebSocket JSON-RPC.

The harness includes four primary tools: frame capture with pixel-level RGB statistics, entity census queries, JSON regression dump with schema validation, and an MCP server for AI-agent-driven verification flows.

All tools are composable — a typical CI run captures a frame, verifies pixel expectations, counts entities, and dumps state in a single orchestrated pass.

## Features

- `civis-verify` — Bevy frame capture with pixel-level validation
- `civis-pixels` — PNG output with RGB histogram and statistics
- `civis-census` — WebSocket JSON-RPC entity count queries
- `civis-dump` — Full simulation state dump with JSON schema validation
- `civis-mcp` — MCP tools server for AI-agent-driven verification
- CI-friendly composable pipeline design

## Usage

```rust
use civis_cli::*;
```

## Architecture

- **PixelStats** — RGB histogram, mean, variance, and golden-image comparison results
- **CensusConfig** — WebSocket endpoint and entity query parameters for census runs
- **DumpValidation** — JSON schema validator for regression state dumps

Each tool connects to a running Civis instance via WebSocket, issues JSON-RPC requests, and returns structured results suitable for CI assertion and reporting.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
