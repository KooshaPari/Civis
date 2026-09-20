# Intent: FR-FR-CORE-009 — Hex grid uses hexx axial coordinates

> Date: 2026-09-20
> FR: FR-FR-CORE-009
> Epic: FR-FR-CORE

## What This FR Captures

The hex-coordinate invariant: engine and render crates use the
`hexx` crate's axial-coordinate representation throughout, with
cube-coordinate conversions used only at the API boundary. The
test in `crates/engine/tests/fr_core_cluster.rs:184` pins the
`PositionAxial → cube → axial` round-trip and asserts the cube
invariant `x + y + z == 0` for a representative grid of axial
coordinates.

## User Intent

A civilization simulation places actors, buildings, and
districts on a hex grid. If two subsystems disagreed about the
coordinate representation (axial vs cube vs offset), they would
glue incorrectly at the boundary and silently misposition
entities. The `hexx` library's axial representation is the
single source of truth.

## Acceptance Signal

- `fr_fr_core_009_axial_cube_roundtrip_holds` passes for the
  representative coordinate set.
- All engine/render crates use `PositionAxial` (or `hexx`'s
  `Axial` type) for hex positions.
- Cube-coordinate derivations satisfy `x + y + z == 0`.

## Implementing Code

- `crates/engine/tests/fr_core_cluster.rs:1` — module header
  lists FR-FR-CORE-009.
- `crates/engine/tests/fr_core_cluster.rs:15` — quoted FR text.
- `crates/engine/tests/fr_core_cluster.rs:184` —
  `fr_fr_core_009_axial_cube_roundtrip_holds` test.

## Test Coverage

- One dedicated test in the test module above covering 20+
  coordinate pairs spanning positive, negative, and asymmetric
  cases.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-fr-core-009-intent.md` |
| Implementing crate | `crates/engine/src/grid.rs` (hexx-backed) |