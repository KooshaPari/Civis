# Intent: FR-CIV-INSTITUTIONS-001 -- Civic institutions persistence

> Date: 2026-09-20
> FR: FR-CIV-INSTITUTIONS-001
> Epic: FR-CIV-INSTITUTIONS

## User Intent

Civic institutions (Temple, Garrison, etc.) and the level-emission
audit trail must survive an archive round-trip so that reloading a
saved world preserves every institution state.

### What This FR Achieves

Adds `institutions: BTreeMap<u32, Vec<civ_institutions::Institution>>`
and `institution_levels_emitted: BTreeSet<(u32, u8, u8)>` to
`WorldState` with `#[serde(default)]` for backwards-compatible
deserialisation; mutates them every tick via `phase_institutions`.

### Product Context

Without durable storage of institutions and level-emission audit
records, reload would silently reset every civic institution,
breaking replay continuity and the legends / strata cascades that
depend on institutional level-ups.

## Acceptance Signal

- Integration test `institutions_buildsites_econfocus_persistence`
  in `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:3`
  passes.
- Persistence-replay coverage in
  `crates/engine/tests/persistence_replay_coverage.rs:12`
  exercises the mirror field.
- Source comment lives at `crates/engine/src/engine.rs:458`
  (declaration) and `crates/engine/src/engine.rs:2182`
  (persistence mirror).

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/engine.rs:458` |
| Code | `crates/engine/src/engine.rs:2182` |
| Test | `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:3` |
| Test | `crates/engine/tests/persistence_replay_coverage.rs:12` |
| Implementing crate | `crates/engine/src/` |
