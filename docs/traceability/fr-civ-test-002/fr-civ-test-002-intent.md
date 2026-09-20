# Intent: FR-CIV-TEST-002 -- civis-mcp integration tests

> Date: 2026-09-19
> FR: FR-CIV-TEST-002
> Epic: FR-CIV-TEST

## User Intent

The product owner requires civis-mcp integration tests as part of the FR-CIV-TEST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines the integration test surface for the
`civis-mcp` tool layer covering the tool registry, JSON schema validation,
`civis_pixels` happy/error/edge paths, `civis_census` error path,
`pixels_for_png` direct lib path, and the `TOOL_NAMES` / router sync invariant.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-TEST-002 contributes to the overall simulation capability by
addressing: external MCP coverage so the tool surface stays stable.

## Acceptance Signal

### Definition of Done

- [x] `crates/civis-mcp/tests/mcp_integration.rs` exists with the integration suite.
- [x] `cargo test -p civis-mcp --test mcp_integration` passes.

### How We Know This FR Is Satisfied

1. Tool registry, schema, and PNG happy/error/edge cases pass.
2. `civis_census` error path returns a descriptive error.
3. `TOOL_NAMES` stays in sync with `tool_router`.

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/civis-mcp/tests/mcp_integration.rs` |
| Implementing crate | `crates/civis-mcp/` |
