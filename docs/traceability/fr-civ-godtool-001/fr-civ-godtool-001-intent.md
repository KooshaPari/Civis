# Intent: FR-CIV-GODTOOL-001 -- `multiply_creatures` god-tool

> Date: 2026-09-19
> FR: FR-CIV-GODTOOL-001
> Epic: FR-CIV-GODTOOL

## User Intent

The Civis MCP god-tool surface needs an entry point for spawning **N**
civilians (where N is 1..=32) within a disk-shaped region, distinct from
the unit-scale `spawn_creature`. This enables design/auditing users to
seed a population spike (post-collapse recovery, plague recovery, miracle
tests) through a single MCP tool call rather than 32 separate
`spawn_creature` round-trips.

### What This FR Achieves

`crates/civis-mcp/src/server.rs` wires the verb:

- `GodAction::MultiplyCreatures` enum variant (line ~149) — tool listing.
- `civis_god_action_multiply_creatures(...)` — the `#[tool_router]` MCP
  function that forwards to `sim.god_action` with verb `multiply_creatures`.
- `GodActionArgs` schema documents a normalised radius and civilian count
  (`1..=32`) shared with `heal` / `bless` / `spawn_creature`.

The verb is consumed by the server-side `sim.god_action` dispatcher
(separate FR) which actually lays out the disk-jittered spawn sites.

## Acceptance Signal

- `cargo build -p civis-mcp` succeeds with `multiply_creatures` exposed.
- The verb is registered in the `GodAction` → `"multiply_creatures"`
  mapping used by `to_params_with_action`.
- Range clamping (1..=32) is enforced server-side; the schema hints that
  in `GodActionArgs::count`.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/civis-mcp/` |
| Enum variant | `crates/civis-mcp/src/server.rs:148` |
| MCP tool fn | `crates/civis-mcp/src/server.rs:1927` |
| Tool registration | `crates/civis-mcp/src/lib.rs:85` |
