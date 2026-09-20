# Intent: FR-CIV-CULTURE-001 — Per-cluster CultureProfile durable state

> FR: FR-CIV-CULTURE-001
> Epic: FR-CIV-CULTURE
> // Covers: FR-CIV-CULTURE-001

## Requirement

Each settlement cluster accumulates a `CultureProfile` (monitoring,
mythic-coherence, uncertainty-reduction). The profile is owned by
`Simulation` and mirrored into the persisted `WorldState` so that v3
archive round-trip recovery is exact. Legacy v3 saves that lack the
field deserialize cleanly because the field defaults to an empty
`BTreeMap`.

## Source

- `crates/engine/src/engine.rs:478` — `Simulation::cluster_cultures`
  field declaration with `#[serde(default)]` and the FR-comment header.
- `crates/engine/src/engine.rs:2190` — `save_state_mirror` copy of the
  field into `WorldState.cluster_cultures`.
- Test: `crates/engine/tests/culture_ideology_aggression_persistence.rs`
  exercises the round-trip path.

## Acceptance

- `cargo test -p civ-engine culture_ideology_aggression_persistence`
  passes.
- A legacy v3 archive that omits `cluster_cultures` deserializes with
  an empty map (no error) and subsequent `save_state_mirror` writes a
  populated map once `phase_culture` runs.
