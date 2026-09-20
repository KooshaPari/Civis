# Intent: FR-CIV-ARCH-D-004 — Culture-id-from-traits determinism

> Date: 2026-09-20
> FR: FR-CIV-ARCH-D-004
> Epic: FR-CIV-ARCH-D

## What This FR Captures

Determinism guarantee for `culture_id_from_traits(traits)` — the
hash of a settlement's cultural-trait vector. Calling the function
twice with the same trait vector must produce the same culture id,
preserving settlement identity across replays.

## User Intent

A settlement's culture id is referenced by diplomacy, art, and
folklore subsystems. Nondeterministic hashing would change which
stories/beliefs get attached to which settlement after a replay,
silently breaking narrative continuity.

## Acceptance Signal

- `cargo test -p build fr_arch_d004_culture_id_is_deterministic`
  passes.

## Implementing Code

- `crates/build/src/tiers.rs:643` — test
  `fr_arch_d004_culture_id_is_deterministic`
- Production code: `culture_id_from_traits` in the same file.

## Test Coverage

- `crates/build/src/tiers.rs:643` (dedicated test, 4-trait vector)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-arch-d-004-intent.md` |
| Implementing crate | `crates/build/src/` |