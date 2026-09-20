# Intent: FR-CIV-CLIENT-006 — Diplomacy action verb

> Date: 2026-09-19
> FR: FR-CIV-CLIENT-006
> Epic: FR-CIV-CLIENT

## User Intent

Clients must be able to issue diplomacy actions against a target faction via
JSON-RPC. Three verbs cover the diplomacy surface: `propose_treaty`,
`declare_war`, `offer_trade`. The MCP server forwards these to `civ-server`'s
`sim.diplomacy_action` method.

## Acceptance Signal

- `crates/server/src/jsonrpc.rs:82` `SimDiplomacyAction` enum variant exists.
- `crates/civis-mcp/src/server.rs:346` `DiplomacyActionKind` matches the verbs.
- `crates/civis-mcp/src/server.rs:2264` MCP tool forwards `sim.diplomacy_action`.
- `// Covers: FR-CIV-CLIENT-006` on the relevant source comments and the spec.
