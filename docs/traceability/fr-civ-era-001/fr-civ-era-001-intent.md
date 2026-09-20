# Intent: FR-CIV-ERA-001 — Era progression persistence

> Date: 2026-09-19
> FR: FR-CIV-ERA-001
> Epic: FR-CIV-ERA

## User Intent

`WorldState` carries an `era_progression: EraProgressionState` field that
survives the archive round-trip. The `phase_era` phase mutates this field
every tick; era gates and per-faction era history persist across save/load.

## Acceptance Signal

- `crates/engine/src/engine.rs:538` field `era_progression` exists with
  `#[serde(default)]` for legacy v3 deserialization.
- `crates/engine/tests/era_emergence_significance_persistence.rs` round-trips
  distinct values byte-for-byte across archive + reload.
- `// Covers: FR-CIV-ERA-001` on the field comment and the persistence test.
