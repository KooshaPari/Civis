---
spec_id: civ-017
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-09
---

# Specification: Civis MCP Server (PR #340 Topic #1)

**Slug**: civ-017-civis-mcp-server | **Epic**: E8 | **Date**: 2026-06-09 | **State**: ACTIVE

## Problem Statement

PR #340 ("feat(extract): wave-1 PR #333 topic #1 — Tools + agileplus-specs +
crates/civis-mcp") landed the `crates/civis-mcp` skeleton as one of the
three non-overlapping new-path subsets. The crate exists; it has no
spec home. This spec is the contract for what `civis-mcp` exposes to
external agents: which JSON-RPC methods it proxies from `civ-server`,
which HTTP routes it forwards to `civ-watch`, and which tools it offers
to MCP-aware clients (so they can interact with a running Civis session
from a Claude / Cursor / dispatch-mcp style host).

## Target Users

- MCP-aware agent hosts (Claude Code, Cursor, dispatch-mcp) consuming
  the `civis-mcp` tool surface
- `civ-server` / `civ-watch` operators who want a thin, well-defined
  proxy in front of the WebSocket and HTTP surfaces
- QA / agent-smoke authors testing the `civis-mcp` tool layer

## Functional Requirements

- [ ] **FR-CIV-MCP-001**: The `civis-mcp` crate SHALL expose an MCP server
  binary (`civis-mcp`) that registers a tool for every JSON-RPC method
  listed in `docs/api/jsonrpc-surface.md` (14 methods today) and SHALL
  forward each tool call to `civ-server` over the existing WebSocket
  surface.
- [ ] **FR-CIV-MCP-002**: The MCP server SHALL register read-only HTTP
  tools for the `civ-watch` endpoints `/terrain`, `/snapshot`, and the
  `sim.snapshot` JSON-RPC equivalent; the tool layer SHALL NOT call any
  mutating HTTP route (`/control/*`) without an explicit `--allow-
  mutations` flag in the CLI.
- [ ] **FR-CIV-MCP-003**: Each tool SHALL be covered by a contract test
  asserting the request envelope shape, the response shape, and the
  error mapping (e.g. JSON-RPC -32601 for unknown methods).
- [ ] **FR-CIV-MCP-004**: The MCP server SHALL be runnable as a stdio
  process AND as a TCP process (`--transport tcp --bind 127.0.0.1:
  <port>`) so it can attach to dispatch-mcp.
- [ ] **FR-CIV-MCP-005**: Configuration SHALL be read from environment
  variables only (`CIVIS_MCP_CIV_SERVER_URL`, `CIVIS_MCP_CIV_WATCH_URL`,
  `CIVIS_MCP_AUTH_TOKEN`); no hardcoded URLs, ports, or secrets.
- [ ] **FR-CIV-MCP-006**: The `agent-smoke.ps1` SHALL add a non-blocking
  `civis-mcp` health check (start the binary, list tools, assert
  non-empty, kill) on the verify path.

## Non-Functional Requirements

- Crate: `crates/civis-mcp/` (321-line `main.rs` skeleton from PR #340)
- MCP SDK: `rmcp` (the modern Rust MCP crate) — see Alternatives
- The MCP server is a **stateless proxy**; it MUST NOT mutate world state
  unless the caller is authorised (default-deny on `/control/*` routes)
- Latency overhead vs direct WS: < 5 ms P99 per tool call
- Determinism: the MCP server is a forwarder; it must not break
  determinism guarantees of the underlying JSON-RPC stream

## Constraints and Dependencies

- Depends on FR-PROTO-001/002 (WebSocket + JSON-RPC) for the proxy path
- Depends on FR-PROTO-003 (client handshake) for the auth flow
- Depends on FR-CIV-VERIFY-001 (`agent-smoke.ps1`) for the gate

## Acceptance Criteria

- [ ] `cargo build -p civis-mcp` succeeds on `main`
- [ ] `cargo test -p civis-mcp` passes the contract test suite
- [ ] `civis-mcp --transport stdio --list-tools` enumerates all 14
  JSON-RPC methods plus the read-only `civ-watch` tools
- [ ] `civis-mcp --transport tcp --bind 127.0.0.1:7777` accepts a
  single client; the smoke test connects, lists tools, disconnects
- [ ] `agent-smoke.ps1` exits 0 with the `civis-mcp` health check
- [ ] No hardcoded URLs / ports / secrets in the crate (audit via
  `findstr` for `127.0.0.1:`, `localhost:`, `http://` patterns)

## Implementation Notes

- The PR #340 `crates/civis-mcp/src/main.rs` is a 321-line skeleton
  with the `rmcp` SDK scaffolded. The contract test surface, the
  `--allow-mutations` flag, and the read-only proxy are the spec
  deltas.
- The MCP server reuses the existing JSON-RPC catalog gate
  (`civis-3d-catalog-check`) — adding a new JSON-RPC method
  automatically adds a new MCP tool.

## Status

| Story | Status |
|-------|--------|
| E8.1 Crate compiles + `rmcp` SDK | implemented (PR #340) |
| E8.2 JSON-RPC tool proxy (14 methods) | Planned |
| E8.3 Read-only `civ-watch` HTTP tools | Planned |
| E8.4 Contract test suite | Planned |
| E8.5 stdio + tcp transports | Planned |
| E8.6 Env-only config | Planned |
| E8.7 `agent-smoke.ps1` health check | Planned |
