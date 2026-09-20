# Intent: FR-CIV-TEST-007 -- civ-server external coverage

> Date: 2026-09-19
> FR: FR-CIV-TEST-007
> Epic: FR-CIV-TEST

## User Intent

The product owner requires civ-server external coverage as part of the FR-CIV-TEST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines external coverage for the civ-server
helper functions that lacked integration tests:
`validate_production_slot`, `save_type_for_name`, `save_category_for_name`,
`mtime_unix_seconds`, `age_label_for_mtime`, `parse_replay_path`, and
`parse_sim_command_action`.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-TEST-007 contributes to the overall simulation capability by
addressing: external validation of the server's save / replay / command helpers.

## Acceptance Signal

### Definition of Done

- [x] `crates/server/tests/server_coverage.rs` exists with the helper coverage tests.
- [x] `cargo test -p civ-server --test server_coverage` passes.

### How We Know This FR Is Satisfied

1. Production slot validation accepts all `PRODUCTION_SLOTS` and rejects unknown.
2. Save type / category lookups round-trip.
3. Replay path and sim command action parsers handle happy + malformed inputs.

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/server/tests/server_coverage.rs` |
| Implementing crate | `crates/server/` |
