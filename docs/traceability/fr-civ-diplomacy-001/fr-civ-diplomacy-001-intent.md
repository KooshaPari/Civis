# Intent: FR-CIV-DIPLOMACY-001 — Diplomacy persistence mirrors

> Date: 2026-09-20
> FR: FR-CIV-DIPLOMACY-001
> Epic: FR-CIV-DIPLOMACY

## What This FR Captures

The "persistence mirror" pattern for diplomacy state. The four
diplomacy surfaces — `faction_relations`, `grief_accumulator`,
`stance_engine`, `deep_diplomacy` — are Simulation-owned (live)
state. The `.civsave.zst` archive only persists the `WorldState`
side. At the end of every tick (`tick()`) and before archive
serialization (`save_state_mirror()`), the live fields are cloned
into `WorldState` so that a save → load → replay round-trip
preserves the exact post-tick diplomacy state.

## User Intent

Without the mirror, replaying a save from an earlier tick would
silently lose every diplomacy mutation made between the save and
the current tick, because the archive reads the snapshot side and
the live side had drifted. The mirror pins both sides to the same
values at save time.

## Acceptance Signal

- `save_state_mirror()` clones the four live diplomacy fields into
  `state.faction_relations`, `state.grief_accumulator`,
  `state.stance_engine`, `state.deep_diplomacy`.
- `tick()` invokes `save_state_mirror()` before
  `replay_log.record_tick(self.state.tick)`.
- `tests/persistence_replay_coverage.rs` confirms diplomacy state
  survives save → load → replay.

## Implementing Code

- `crates/engine/src/engine.rs:2168` — `save_state_mirror` mirrors
  the four diplomacy surfaces (and several others).
- `crates/engine/src/engine.rs:2343` — `tick` invokes the mirror
  before recording the tick.

## Test Coverage

- `crates/engine/tests/diplomacy_flow.rs:20` — full diplomacy
  flow (faction formation, treaty propagation).
- `crates/engine/tests/persistence_replay_coverage.rs:15` —
  `deep_diplomacy.faction_resources` replay round-trip test.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-diplomacy-001-intent.md` |
| Implementing crate | `crates/engine/src/` |