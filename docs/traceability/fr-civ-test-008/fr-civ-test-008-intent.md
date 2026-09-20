# Intent: FR-CIV-TEST-008 -- civ-emergence-metrics + civis-mcp coverage

> Date: 2026-09-19
> FR: FR-CIV-TEST-008
> Epic: FR-CIV-TEST

## User Intent

The product owner requires civ-emergence-metrics + civis-mcp coverage as part of the FR-CIV-TEST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines external coverage tests for two crates:

- `civ-emergence-metrics` — `BranchingRegime::label()` (all five arms with
  stable wire labels), `BranchingLedger::closed_total()` (monotonic counter),
  `JointHistogram::rows()` / `cols()` dimension accessors used by the
  dashboard renderer.
- `civis-mcp` — `HARNESS_VERSION` is non-empty semver, and
  `census_config_with_url()` returns a valid non-empty URL even with no env vars.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-TEST-008 contributes to the overall simulation capability by
addressing: external validation of branch-regime wire labels, joint histogram
dimensions, harness version, and census URL wiring.

## Acceptance Signal

### Definition of Done

- [x] `crates/civ-emergence-metrics/tests/emergence_coverage.rs` exists.
- [x] `crates/civis-mcp/tests/mcp_coverage.rs` exists.
- [x] Both `cargo test` invocations pass.

### How We Know This FR Is Satisfied

1. BranchingRegime labels are stable wire strings.
2. JointHistogram dimensions are accessible.
3. HARNESS_VERSION is non-empty semver.
4. `census_config_with_url()` returns a non-empty URL.

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/civ-emergence-metrics/tests/emergence_coverage.rs` |
| Tests | `crates/civis-mcp/tests/mcp_coverage.rs` |
| Implementing crate | `crates/civ-emergence-metrics/`, `crates/civis-mcp/` |
