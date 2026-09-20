# Intent: FR-CIV-SERVER-003 — Research JSON-RPC API

> Date: 2026-09-20
> FR: FR-CIV-SERVER-003
> Epic: FR-CIV-SERVER

## What This FR Captures

The research-tech JSON-RPC surface on civ-server: two methods
(`sim.queue_research`, `sim.tech_state`) plus their dispatch
plumbing and test coverage. `sim.queue_research` accepts a tech
name from a closed set (12 known names), emits a
`DispatchEffect::QueueResearch { tech }`, and rejects unknowns
with `INVALID_PARAMS`. `sim.tech_state` returns the available
list plus any stub flag for in-progress techs.

The MCP bridge in `civis-mcp` exposes both as `civis_research_queue`
and `civis_tech_state` tools that forward to the same RPC
endpoints.

## User Intent

Server clients (the web dashboard, the Godot/Unreal clients, the
MCP bridge used by agentic UIs) all need a stable RPC to queue a
research target and to read the current research state. Centralizing
the tech whitelist on the server prevents clients from inventing
unknown tech names that would silently no-op server-side.

## Acceptance Signal

- `dispatch_queue_research_valid_tech_accepted` —
  `sim.queue_research` with `pottery` returns
  `DispatchEffect::QueueResearch { tech: "pottery" }` and a result
  payload of `{ "queued": "pottery" }`.
- `dispatch_queue_research_unknown_tech_rejected` —
  unknown tech returns `INVALID_PARAMS` and `DispatchEffect::None`.
- `dispatch_tech_state_returns_available_list` — `sim.tech_state`
  returns the available list (with stub flag).
- MCP `civis_research_queue` / `civis_tech_state` forward to the
  same endpoints.

## Implementing Code

- `crates/server/src/jsonrpc.rs:84` — `SimQueueResearch` /
  `SimTechState` enum variants w/ FR ref.
- `crates/server/src/jsonrpc.rs:4957` — accepted-tech test.
- `crates/server/src/jsonrpc.rs:4996` — rejected-tech test.
- `crates/server/src/jsonrpc.rs:5032` — `tech_state` test.
- `crates/civis-mcp/src/server.rs:2287` — `civis_research_queue`.
- `crates/civis-mcp/src/server.rs:2310` — `civis_tech_state`.

## Test Coverage

- Three dedicated tests in `crates/server/src/jsonrpc.rs` test
  module.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-server-003-intent.md` |
| Implementing crate | `crates/server/src/`, `crates/civis-mcp/src/` |