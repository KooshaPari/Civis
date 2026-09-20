# Intent: FR-CIV-GOV-003 — L1 → L2 institution upgrade (one-shot)

> FR: FR-CIV-GOV-003
> Epic: FR-CIV-GOV
> // Covers: FR-CIV-GOV-003

## Requirement

When a settlement crosses the L2 population threshold for an institution
kind, an `Upgraded` event fires exactly once per
`(settlement_id, kind, level)` triple. Subsequent population dips below
the threshold and rebounds do not produce duplicate upgrade events.

`TEMPLE_L2_POPULATION = 200`, `GARRISON_L2_POPULATION = 400` are the
population thresholds; the monotonic set
`Simulation::institution_levels_emitted` is what enforces one-shot
semantics.

## Source

- `crates/civ-institutions/src/lib.rs:38` — module doc covers FR-CIV-GOV-003
  and lists `TEMPLE_L2_POPULATION` / `GARRISON_L2_POPULATION`.
- `crates/engine/src/engine.rs:856` — `institutions` field comments.
- `crates/engine/src/engine.rs:865` — `institution_levels_emitted` field
  comments.
- Test: `crates/engine/tests/fr_civ_gov_institutions.rs`.

## Acceptance

- `cargo test -p civ-engine fr_civ_gov_institutions` passes.
- Crossing `TEMPLE_L2_POPULATION` (resp. `GARRISON_L2_POPULATION`)
  emits a single `Upgraded` event per settlement / kind, idempotent
  across population dips.
