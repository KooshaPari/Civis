# Intent: FR-CIV-TEST-009 -- protocol-3d wire coverage

> Date: 2026-09-19
> FR: FR-CIV-TEST-009
> Epic: FR-CIV-TEST

## User Intent

The product owner requires protocol-3d wire coverage as part of the FR-CIV-TEST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines external coverage for `protocol-3d` wire
types: `CivilianStateEntry` serde round-trip with legacy health default,
`BuildingDiffFrame` serde with `skip_serializing_if` on empty buildings,
`quantize_axis` / `dequantize_axis` round-trip at non-zero origin, and the
`is_frame3d_bundle` vs `is_frame3d_binary` magic boundary.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-TEST-009 contributes to the overall simulation capability by
addressing: external validation of the protocol wire shape and frame magic.

## Acceptance Signal

### Definition of Done

- [x] `crates/protocol-3d/tests/protocol_coverage.rs` exists.
- [x] `cargo test -p protocol-3d --test protocol_coverage` passes.

### How We Know This FR Is Satisfied

1. CivilianStateEntry round-trips and legacy health defaults to a sane value.
2. BuildingDiffFrame serializes cleanly when buildings are empty.
3. `quantize_axis` / `dequantize_axis` round-trips at non-zero origin.
4. `is_frame3d_bundle` / `is_frame3d_binary` correctly disambiguate the magic bytes.

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/protocol-3d/tests/protocol_coverage.rs` |
| Implementing crate | `crates/protocol-3d/` |
