# Intent: FR-CIV-GOV-010 — Social-mood phase (superseded home)

> FR: FR-CIV-GOV-010
> Epic: FR-CIV-GOV
> // Covers: FR-CIV-GOV-010

## Requirement

`Simulation::phase_social_mood` derives per-settlement mood from
food_per_capita, housing surplus vs population, and inverse crime
pressure, and adds institution bonuses (Temple + Garrison). Mood is
saturated to `[-200, 200]`.

The implementation lives in
`crates/engine/src/engine/social_settlement_phases.rs` (and the dormant
stub block in `crates/engine/src/dormant_phases.rs`). The
`engine.rs:3431` comment block documents the superset-merge that
moved the canonical home out of the legacy stubs and into the primary
`impl Simulation` block so the phase_* methods can be `fn` rather
than `pub fn`.

## Source

- `crates/engine/src/engine.rs:3431` — superset-merge note referencing
  `phase_social_mood (FR-CIV-GOV-010)`.
- `crates/engine/src/engine/social_settlement_phases.rs` — the actual
  phase implementation.
- Test: `crates/engine/tests/fr_civ_gov_mood.rs`.

## Acceptance

- `cargo test -p civ-engine fr_civ_gov_mood` passes.
- The primary `impl Simulation` defines exactly one `phase_social_mood`
  body; no duplicate symbols.
