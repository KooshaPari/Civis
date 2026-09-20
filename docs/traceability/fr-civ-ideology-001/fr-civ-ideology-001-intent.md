# Intent: FR-CIV-IDEOLOGY-001 — Faction ideology persistence

> Date: 2026-09-19
> FR: FR-CIV-IDEOLOGY-001
> Epic: FR-CIV-IDEOLOGY

## User Intent

`WorldState` carries a `faction_ideologies: BTreeMap<u32, FactionIdeologyState>`
field that survives the archive round-trip. The `phase_ideology` phase
mutates this field every tick; per-faction cultural doctrine flags persist
across save/load.

## Acceptance Signal

- `crates/engine/src/engine.rs:486` field `faction_ideologies` exists with
  `#[serde(default)]` for legacy v3 deserialization.
- `crates/engine/tests/culture_ideology_aggression_persistence.rs` inserts
  distinct values, advances one tick to fire `save_state_mirror`, then
  asserts the field round-trips exactly.
- `// Covers: FR-CIV-IDEOLOGY-001` on the field comment and the persistence test.
